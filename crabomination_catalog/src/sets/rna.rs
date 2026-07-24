//! Ravnica Allegiance (RNA) — 2019. Commons/uncommons on existing primitives.
//! Tests in `classic_sets/rna`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::card::SelectionRequirement as R;
use crate::effect::shortcut::{
    adapt, afterlife, deal, draw, etb, etb_scry, on_attack, riot, spectacle, target_filtered,
};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, RevealMissDest, Selector,
    StaticEffect, ZoneDest,
};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

fn body(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32, ct: Vec<CreatureType>, kw: Vec<Keyword>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: creatures(ct),
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

/// Catacomb Crocodile — {4}{B} 3/7 Crocodile.
pub fn catacomb_crocodile() -> CardDefinition {
    body("Catacomb Crocodile", cost(&[generic(4), b()]), 3, 7, vec![CreatureType::Crocodile], vec![])
}

/// Azorius Knight-Arbiter — {3}{W}{U} 2/5 Human Knight. Vigilance; can't be
/// blocked.
pub fn azorius_knight_arbiter() -> CardDefinition {
    body("Azorius Knight-Arbiter", cost(&[generic(3), w(), u()]), 2, 5, vec![CreatureType::Human, CreatureType::Knight], vec![Keyword::Vigilance, Keyword::Unblockable])
}

/// Carrion Imp — {3}{B} 2/3 Imp with flying. ETB may exile a creature card from
/// a graveyard; if you do, gain 2 life.
pub fn carrion_imp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile target creature card from a graveyard; gain 2 life.".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::Creature.and(R::InGraveyard)), to: ZoneDest::Exile },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ])),
        })],
        ..body("Carrion Imp", cost(&[generic(3), b()]), 2, 3, vec![CreatureType::Imp], vec![Keyword::Flying])
    }
}

/// Civic Stalwart — {3}{W} 3/3 Elephant Soldier. ETB creatures you control get
/// +1/+1 until end of turn.
pub fn civic_stalwart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..body("Civic Stalwart", cost(&[generic(3), w()]), 3, 3, vec![CreatureType::Elephant, CreatureType::Soldier], vec![])
    }
}

/// Blade Juggler — {4}{B} 3/2 Human Rogue with Spectacle {2}{B}. ETB deals 1
/// damage to you and you draw a card.
pub fn blade_juggler() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(spectacle(cost(&[generic(2), b()]))),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            deal(1, Selector::You),
            draw(1),
        ]))],
        ..body("Blade Juggler", cost(&[generic(4), b()]), 3, 2, vec![CreatureType::Human, CreatureType::Rogue], vec![])
    }
}

/// Devkarin Dissident — {1}{G} 2/2 Elf Warrior. {4}{G}: +2/+2 until end of turn.
pub fn devkarin_dissident() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body("Devkarin Dissident", cost(&[generic(1), g()]), 2, 2, vec![CreatureType::Elf, CreatureType::Warrior], vec![])
    }
}

/// Passwall Adept — {1}{U} 1/3 Human Wizard. {2}{U}: target creature can't be
/// blocked this turn.
pub fn passwall_adept() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body("Passwall Adept", cost(&[generic(1), u()]), 1, 3, vec![CreatureType::Human, CreatureType::Wizard], vec![])
    }
}

/// Rakdos Firewheeler — {B}{B}{R}{R} 4/3 Human Rogue. ETB deals 2 to target
/// opponent and 2 to up to one target creature or planeswalker.
pub fn rakdos_firewheeler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer },
                    amount: Value::Const(2),
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature.or(R::HasCardType(CardType::Planeswalker)) },
                    amount: Value::Const(2),
                },
            ])),
        })],
        ..body("Rakdos Firewheeler", cost(&[b(), b(), r(), r()]), 4, 3, vec![CreatureType::Human, CreatureType::Rogue], vec![])
    }
}

/// Gyre Engineer — {1}{G}{U} 1/1 Vedalken Wizard. {T}: Add {G}{U}. Whenever you
/// activate an adapt ability, untap Gyre Engineer.
pub fn gyre_engineer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Blue]) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AdaptAbilityActivated, EventScope::YourControl),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..body("Gyre Engineer", cost(&[generic(1), g(), u()]), 1, 1, vec![CreatureType::Vedalken, CreatureType::Wizard], vec![])
    }
}

/// Bring to Trial — {2}{W} Sorcery. Exile target creature with power 4 or
/// greater.
pub fn bring_to_trial() -> CardDefinition {
    CardDefinition {
        name: "Bring to Trial",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::Creature.and(R::PowerAtLeast(4))),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Burn Bright — {2}{R} Instant. Creatures you control get +2/+0 until end of
/// turn.
pub fn burn_bright() -> CardDefinition {
    CardDefinition {
        name: "Burn Bright",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Applied Biomancy — {G}{U} Instant. Choose one or both — target creature gets
/// +1/+1 until end of turn; and/or return target creature to its owner's hand.
pub fn applied_biomancy() -> CardDefinition {
    CardDefinition {
        name: "Applied Biomancy",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Arrester's Zeal — {W} Instant. Target creature gets +2/+2 until end of turn.
/// Addendum — if cast during your main phase, it also gains flying.
pub fn arresters_zeal() -> CardDefinition {
    CardDefinition {
        name: "Arrester's Zeal",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Arrester's Admonition — {2}{U} Instant. Return target creature to its owner's
/// hand. Addendum — if cast during your main phase, draw a card.
pub fn arresters_admonition() -> CardDefinition {
    CardDefinition {
        name: "Arrester's Admonition",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Ironshell Beetle — {1}{G} 1/1 Insect. ETB put a +1/+1 counter on target
/// creature.
pub fn ironshell_beetle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..body("Ironshell Beetle", cost(&[generic(1), g()]), 1, 1, vec![CreatureType::Insect], vec![])
    }
}

/// Vizkopa Vampire — {2}{W/B} 3/1 Vampire with lifelink.
pub fn vizkopa_vampire() -> CardDefinition {
    body("Vizkopa Vampire", cost(&[generic(2), hybrid(Color::White, Color::Black)]), 3, 1, vec![CreatureType::Vampire], vec![Keyword::Lifelink])
}

/// Rubblebelt Recluse — {4}{R} 6/5 Ogre Berserker that attacks each combat if
/// able.
pub fn rubblebelt_recluse() -> CardDefinition {
    body("Rubblebelt Recluse", cost(&[generic(4), r()]), 6, 5, vec![CreatureType::Ogre, CreatureType::Berserker], vec![Keyword::MustAttack])
}

/// Rakdos Trumpeter — {1}{B} 1/3 Human Shaman with menace. {3}{R}: +2/+0 until
/// end of turn.
pub fn rakdos_trumpeter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            effect: Effect::PumpPT { what: Selector::This, power: Value::Const(2), toughness: Value::ZERO, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..body("Rakdos Trumpeter", cost(&[generic(1), b()]), 1, 3, vec![CreatureType::Human, CreatureType::Shaman], vec![Keyword::Menace])
    }
}

/// Griffin Protector — {3}{W} 2/3 Griffin with flying. Whenever another creature
/// you control enters, it gets +1/+1 until end of turn.
pub fn griffin_protector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::OtherThanSource),
            }),
            effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
        }],
        ..body("Griffin Protector", cost(&[generic(3), w()]), 2, 3, vec![CreatureType::Griffin], vec![Keyword::Flying])
    }
}

/// A vanilla token creature body of `colors`, P/T, and creature types.
fn token(name: &'static str, colors: Vec<Color>, p: i32, t: i32, ct: Vec<CreatureType>, kw: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords: kw,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: creatures(ct),
        ..Default::default()
    }
}

/// Tithe Taker — {1}{W} 2/1 Human Soldier with Afterlife 1. During your turn,
/// opponents' spells and non-mana abilities cost {1} more.
pub fn tithe_taker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(1)],
        static_abilities: vec![StaticAbility {
            description: "During your turn, opponents' spells and non-mana abilities cost {1} more.",
            effect: StaticEffect::OpponentActivityCostsMoreOnYourTurn { amount: 1 },
        }],
        ..body("Tithe Taker", cost(&[generic(1), w()]), 2, 1, vec![CreatureType::Human, CreatureType::Soldier], vec![])
    }
}

/// Imperious Oligarch — {W}{B} 2/1 Human Cleric with vigilance and Afterlife 1.
pub fn imperious_oligarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(1)],
        ..body("Imperious Oligarch", cost(&[w(), b()]), 2, 1, vec![CreatureType::Human, CreatureType::Cleric], vec![Keyword::Vigilance])
    }
}

/// Rampaging Rendhorn — {4}{G} 4/4 Beast with Riot.
pub fn rampaging_rendhorn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        ..body("Rampaging Rendhorn", cost(&[generic(4), g()]), 4, 4, vec![CreatureType::Beast], vec![])
    }
}

/// Spear Spewer — {R} 0/2 Goblin Warrior with defender. {T}: deal 1 damage to
/// each player.
pub fn spear_spewer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
            ..Default::default()
        }],
        ..body("Spear Spewer", cost(&[r()]), 0, 2, vec![CreatureType::Goblin, CreatureType::Warrior], vec![Keyword::Defender])
    }
}

/// Vindictive Vampire — {3}{B} 2/3 Vampire. Whenever another creature you
/// control dies, deal 1 damage to each opponent and gain 1 life.
pub fn vindictive_vampire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::OtherThanSource,
            }),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..body("Vindictive Vampire", cost(&[generic(3), b()]), 2, 3, vec![CreatureType::Vampire], vec![])
    }
}

/// Sauroform Hybrid — {1}{G} 2/2 Human Lizard Warrior. {4}{G}{G}: Adapt 4.
pub fn sauroform_hybrid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), g()]),
            effect: adapt(4),
            ..Default::default()
        }],
        ..body("Sauroform Hybrid", cost(&[generic(1), g()]), 2, 2, vec![CreatureType::Human, CreatureType::Lizard, CreatureType::Warrior], vec![])
    }
}

/// Skitter Eel — {3}{U} 3/3 Fish Crab. {2}{U}: Adapt 2.
pub fn skitter_eel() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: adapt(2),
            ..Default::default()
        }],
        ..body("Skitter Eel", cost(&[generic(3), u()]), 3, 3, vec![CreatureType::Fish, CreatureType::Crab], vec![])
    }
}

/// Titanic Brawl — {1}{G} Instant. Costs {1} less if it targets a creature you
/// control with a +1/+1 counter. Target creature you control fights a creature
/// you don't control.
pub fn titanic_brawl() -> CardDefinition {
    CardDefinition {
        name: "Titanic Brawl",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_cost_if_target: Some((
            R::Creature.and(R::ControlledByYou).and(R::WithCounter(CounterType::PlusOnePlusOne)),
            cost(&[generic(1)]),
        )),
        effect: Effect::Fight {
            attacker: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
        },
        ..Default::default()
    }
}

/// Rakdos Roustabout — {1}{B}{R} 3/2 Ogre Warrior. Whenever it becomes blocked,
/// it deals 1 damage to the player it's attacking.
pub fn rakdos_roustabout() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE },
        }],
        ..body("Rakdos Roustabout", cost(&[generic(1), b(), r()]), 3, 2, vec![CreatureType::Ogre, CreatureType::Warrior], vec![])
    }
}

/// Consign to the Pit — {5}{B} Sorcery. Destroy target creature and deal 2
/// damage to that creature's controller.
pub fn consign_to_the_pit() -> CardDefinition {
    CardDefinition {
        name: "Consign to the Pit",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))), amount: Value::Const(2) },
            Effect::Destroy { what: target_filtered(R::Creature) },
        ]),
        ..Default::default()
    }
}

/// Scorchmark — {1}{R} Instant. Deal 2 damage to target creature; if it would
/// die this turn, exile it instead.
pub fn scorchmark() -> CardDefinition {
    CardDefinition {
        name: "Scorchmark",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: target_filtered(R::Creature) },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Senate Guildmage — {W}{U} 2/2 Human Wizard. {W}, {T}: gain 2 life. {U}, {T}:
/// draw a card, then discard a card.
pub fn senate_guildmage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                tap_cost: true,
                effect: Effect::Seq(vec![draw(1), Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }]),
                ..Default::default()
            },
        ],
        ..body("Senate Guildmage", cost(&[w(), u()]), 2, 2, vec![CreatureType::Human, CreatureType::Wizard], vec![])
    }
}

/// Undercity Scavenger — {3}{B} 3/3 Ogre Warrior. ETB you may sacrifice another
/// creature; if you do, put two +1/+1 counters on it, then scry 2.
pub fn undercity_scavenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Sacrifice another creature: two +1/+1 counters and scry 2.".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature.and(R::OtherThanSource) },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            ])),
        })],
        ..body("Undercity Scavenger", cost(&[generic(3), b()]), 3, 3, vec![CreatureType::Ogre, CreatureType::Warrior], vec![])
    }
}

/// Gatebreaker Ram — {2}{G} 2/2 Sheep. +1/+1 for each Gate you control; while
/// you control two or more Gates it has vigilance and trample.
pub fn gatebreaker_ram() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Gets +1/+1 for each Gate you control.",
                effect: StaticEffect::PumpSelfByControlledPermanents {
                    filter: R::HasLandType(LandType::Gate),
                    per_power: 1,
                    per_toughness: 1,
                },
            },
            StaticAbility {
                description: "While you control two or more Gates, has vigilance and trample.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(R::HasLandType(LandType::Gate).and(R::ControlledByYou)),
                        n: Value::Const(2),
                    },
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Vigilance, Keyword::Trample],
                },
            },
        ],
        ..body("Gatebreaker Ram", cost(&[generic(2), g()]), 2, 2, vec![CreatureType::Sheep], vec![])
    }
}

/// Feral Maaka — {1}{R} 2/2 Cat.
pub fn feral_maaka() -> CardDefinition {
    body("Feral Maaka", cost(&[generic(1), r()]), 2, 2, vec![CreatureType::Cat], vec![])
}

/// Rubble Slinger — {2}{R/G} 2/3 Human Warrior with reach.
pub fn rubble_slinger() -> CardDefinition {
    body("Rubble Slinger", cost(&[generic(2), hybrid(Color::Red, Color::Green)]), 2, 3, vec![CreatureType::Human, CreatureType::Warrior], vec![Keyword::Reach])
}

/// Watchful Giant — {5}{W} 3/6 Giant Soldier. ETB create a 1/1 white Human.
pub fn watchful_giant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: token("Human", vec![Color::White], 1, 1, vec![CreatureType::Human], vec![]),
        })],
        ..body("Watchful Giant", cost(&[generic(5), w()]), 3, 6, vec![CreatureType::Giant, CreatureType::Soldier], vec![])
    }
}

/// Faerie Duelist — {1}{U} 1/2 Faerie Rogue with flash and flying. ETB target
/// creature an opponent controls gets -2/-0 until end of turn.
pub fn faerie_duelist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..body("Faerie Duelist", cost(&[generic(1), u()]), 1, 2, vec![CreatureType::Faerie, CreatureType::Rogue], vec![])
    }
}

/// Coral Commando — {2}{U} 3/2 Merfolk Warrior.
pub fn coral_commando() -> CardDefinition {
    body("Coral Commando", cost(&[generic(2), u()]), 3, 2, vec![CreatureType::Merfolk, CreatureType::Warrior], vec![])
}

/// Windstorm Drake — {4}{U} 3/3 Drake with flying. Other creatures you control
/// with flying get +1/+0.
pub fn windstorm_drake() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with flying get +1/+0.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasKeyword(Keyword::Flying).and(R::OtherThanSource),
                power: 1,
                toughness: 0,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body("Windstorm Drake", cost(&[generic(4), u()]), 3, 3, vec![CreatureType::Drake], vec![Keyword::Flying])
    }
}

/// Drill Bit — {2}{B} Sorcery with Spectacle {B}. Target player reveals their
/// hand; you choose a nonland card; that player discards it.
pub fn drill_bit() -> CardDefinition {
    CardDefinition {
        name: "Drill Bit",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        alternative_cost: Some(spectacle(cost(&[b()]))),
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::Nonland,
        },
        ..Default::default()
    }
}

/// Burning-Tree Vandal — {2}{R} 2/1 Human Rogue with Riot. Whenever it attacks,
/// you may discard a card; if you do, draw a card.
pub fn burning_tree_vandal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            riot(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Discard a card, then draw a card.".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                        draw(1),
                    ])),
                },
            },
        ],
        ..body("Burning-Tree Vandal", cost(&[generic(2), r()]), 2, 1, vec![CreatureType::Human, CreatureType::Rogue], vec![])
    }
}

/// Ghor-Clan Wrecker — {3}{R} 2/2 Human Warrior with Riot and menace.
pub fn ghor_clan_wrecker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        ..body("Ghor-Clan Wrecker", cost(&[generic(3), r()]), 2, 2, vec![CreatureType::Human, CreatureType::Warrior], vec![Keyword::Menace])
    }
}

/// Sprouting Renewal — {2}{G} Sorcery with convoke. Choose one — create a 2/2
/// green-and-white Elf Knight with vigilance; or destroy target artifact or
/// enchantment.
pub fn sprouting_renewal() -> CardDefinition {
    CardDefinition {
        name: "Sprouting Renewal",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Convoke],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: token("Elf Knight", vec![Color::Green, Color::White], 2, 2, vec![CreatureType::Elf, CreatureType::Knight], vec![Keyword::Vigilance]),
                },
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Open the Gates — {G} Sorcery. Search your library for a basic land or Gate
/// card, reveal it, put it into your hand, then shuffle.
pub fn open_the_gates() -> CardDefinition {
    CardDefinition {
        name: "Open the Gates",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand.or(R::HasLandType(LandType::Gate)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Cindervines — {R}{G} Enchantment. Whenever an opponent casts a noncreature
/// spell, deal 1 damage to that player. {1}, Sacrifice this: destroy target
/// artifact or enchantment and deal 2 damage to that permanent's controller.
pub fn cindervines() -> CardDefinition {
    CardDefinition {
        name: "Cindervines",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Not(Box::new(R::Creature)),
            }),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))), amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))), amount: Value::Const(2) },
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Summary Judgment — {1}{W} Instant. Deal 3 damage to target tapped creature;
/// Addendum — 5 damage instead if cast during your main phase.
pub fn summary_judgment() -> CardDefinition {
    CardDefinition {
        name: "Summary Judgment",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::YourMainPhase,
            then: Box::new(Effect::DealDamage { to: target_filtered(R::Creature.and(R::Tapped)), amount: Value::Const(5) }),
            else_: Box::new(Effect::DealDamage { to: target_filtered(R::Creature.and(R::Tapped)), amount: Value::Const(3) }),
        },
        ..Default::default()
    }
}

/// Haazda Officer — {2}{W} 3/2 Human Soldier. ETB target creature you control
/// gets +1/+1 until end of turn.
pub fn haazda_officer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..body("Haazda Officer", cost(&[generic(2), w()]), 3, 2, vec![CreatureType::Human, CreatureType::Soldier], vec![])
    }
}

/// Twilight Panther — {W} 1/2 Cat Spirit. {B}: gains deathtouch until end of turn.
pub fn twilight_panther() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..body("Twilight Panther", cost(&[w()]), 1, 2, vec![CreatureType::Cat, CreatureType::Spirit], vec![])
    }
}

/// Vedalken Mesmerist — {1}{U} 2/1 Vedalken Wizard. Whenever it attacks, target
/// creature an opponent controls gets -2/-0 until end of turn.
pub fn vedalken_mesmerist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..body("Vedalken Mesmerist", cost(&[generic(1), u()]), 2, 1, vec![CreatureType::Vedalken, CreatureType::Wizard], vec![])
    }
}

/// Chillbringer — {4}{U} 3/3 Elemental with flying. ETB tap target creature an
/// opponent controls; it doesn't untap during its controller's next untap step.
pub fn chillbringer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
        ]))],
        ..body("Chillbringer", cost(&[generic(4), u()]), 3, 3, vec![CreatureType::Elemental], vec![Keyword::Flying])
    }
}

/// Grotesque Demise — {2}{B} Instant. Exile target creature with power 3 or less.
pub fn grotesque_demise() -> CardDefinition {
    CardDefinition {
        name: "Grotesque Demise",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move { what: target_filtered(R::Creature.and(R::PowerAtMost(3))), to: ZoneDest::Exile },
        ..Default::default()
    }
}

/// Noxious Groodion — {2}{B} 2/2 Beast with deathtouch.
pub fn noxious_groodion() -> CardDefinition {
    body("Noxious Groodion", cost(&[generic(2), b()]), 2, 2, vec![CreatureType::Beast], vec![Keyword::Deathtouch])
}

/// Cavalcade of Calamity — {1}{R} Enchantment. Whenever a creature you control
/// with power 1 or less attacks, deal 1 damage to the player it's attacking.
pub fn cavalcade_of_calamity() -> CardDefinition {
    CardDefinition {
        name: "Cavalcade of Calamity",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::PowerAtMost(1)),
            }),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Rubble Reading — {3}{R} Sorcery. Destroy target land, then scry 2.
pub fn rubble_reading() -> CardDefinition {
    CardDefinition {
        name: "Rubble Reading",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Land) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Regenesis — {3}{G}{G} Instant. Return up to two target permanent cards from
/// your graveyard to your hand.
pub fn regenesis() -> CardDefinition {
    CardDefinition {
        name: "Regenesis",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::InGraveyard.and(R::Not(Box::new(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))))),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
        },
        ..Default::default()
    }
}

/// Steeple Creeper — {2}{G} 4/2 Frog Snake. {3}{U}: gains flying until end of turn.
pub fn steeple_creeper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..body("Steeple Creeper", cost(&[generic(2), g()]), 4, 2, vec![CreatureType::Frog, CreatureType::Snake], vec![])
    }
}

/// Gruul Beastmaster — {3}{G} 2/2 Human Shaman with Riot. Whenever it attacks,
/// another target creature you control gets +X/+0 until end of turn, where X is
/// this creature's power.
pub fn gruul_beastmaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            riot(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                    power: Value::PowerOf(Box::new(Selector::This)),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..body("Gruul Beastmaster", cost(&[generic(3), g()]), 2, 2, vec![CreatureType::Human, CreatureType::Shaman], vec![])
    }
}

/// Trollbred Guardian — {4}{G} 5/5 Troll Frog Warrior. {2}{G}: Adapt 2. Each
/// creature you control with a +1/+1 counter has trample.
pub fn trollbred_guardian() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: adapt(2),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter has trample.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::WithCounter(CounterType::PlusOnePlusOne),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Trample],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body("Trollbred Guardian", cost(&[generic(4), g()]), 5, 5, vec![CreatureType::Troll, CreatureType::Frog, CreatureType::Warrior], vec![])
    }
}

/// Loxodon Restorer — {4}{W}{W} 3/4 Elephant Cleric with convoke. ETB gain 4 life.
pub fn loxodon_restorer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(4) })],
        ..body("Loxodon Restorer", cost(&[generic(4), w(), w()]), 3, 4, vec![CreatureType::Elephant, CreatureType::Cleric], vec![])
    }
}

/// Syndicate Messenger — {3}{W} 2/3 Bird with flying and Afterlife 1.
pub fn syndicate_messenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(1)],
        ..body("Syndicate Messenger", cost(&[generic(3), w()]), 2, 3, vec![CreatureType::Bird], vec![Keyword::Flying])
    }
}

/// Prying Eyes — {4}{U}{U} Instant. Draw four cards, then discard two cards.
pub fn prying_eyes() -> CardDefinition {
    CardDefinition {
        name: "Prying Eyes",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            draw(4),
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
        ]),
        ..Default::default()
    }
}

// ══════════════════════════════════════════════════════════════════════════
// Batch 4 (2026-07-24): Locket cycle, adapt/scry creatures, spectacle, spells
// ══════════════════════════════════════════════════════════════════════════

/// The RNA guild Locket cycle — {3} artifacts that tap for one of two colors
/// and sacrifice (paying four hybrid mana) to draw two cards.
fn locket(name: &'static str, c1: Color, c2: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![c1, c2], Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[hybrid(c1, c2), hybrid(c1, c2), hybrid(c1, c2), hybrid(c1, c2)]),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
pub fn azorius_locket() -> CardDefinition { locket("Azorius Locket", Color::White, Color::Blue) }
pub fn orzhov_locket() -> CardDefinition { locket("Orzhov Locket", Color::White, Color::Black) }
pub fn rakdos_locket() -> CardDefinition { locket("Rakdos Locket", Color::Black, Color::Red) }
pub fn gruul_locket() -> CardDefinition { locket("Gruul Locket", Color::Red, Color::Green) }
pub fn simic_locket() -> CardDefinition { locket("Simic Locket", Color::Green, Color::Blue) }

/// Aeromunculus — {1}{G}{U} 2/3 Homunculus Mutant with flying. {2}{G}{U}: Adapt 1.
pub fn aeromunculus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), u()]),
            effect: adapt(1),
            ..Default::default()
        }],
        ..body("Aeromunculus", cost(&[generic(1), g(), u()]), 2, 3, vec![CreatureType::Homunculus, CreatureType::Mutant], vec![Keyword::Flying])
    }
}

/// Sage's Row Savant — {1}{U} 2/1 Vedalken Wizard. ETB scry 2.
pub fn sages_row_savant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_scry(2)],
        ..body("Sage's Row Savant", cost(&[generic(1), u()]), 2, 1, vec![CreatureType::Vedalken, CreatureType::Wizard], vec![])
    }
}

/// Senate Griffin — {2}{W/U}{W/U} 3/2 Griffin with flying. ETB scry 1.
pub fn senate_griffin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_scry(1)],
        ..body("Senate Griffin", cost(&[generic(2), hybrid(Color::White, Color::Blue), hybrid(Color::White, Color::Blue)]), 3, 2, vec![CreatureType::Griffin], vec![Keyword::Flying])
    }
}

/// Sylvan Brushstrider — {2}{G} 3/2 Beast. ETB gain 2 life.
pub fn sylvan_brushstrider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(2) })],
        ..body("Sylvan Brushstrider", cost(&[generic(2), g()]), 3, 2, vec![CreatureType::Beast], vec![])
    }
}

/// Wrecking Beast — {5}{G}{G} 6/6 Beast with riot and trample.
pub fn wrecking_beast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        ..body("Wrecking Beast", cost(&[generic(5), g(), g()]), 6, 6, vec![CreatureType::Beast], vec![Keyword::Trample])
    }
}

/// Thirsting Shade — {B} 1/1 Shade with lifelink. {2}{B}: +1/+1 until end of turn.
pub fn thirsting_shade() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..body("Thirsting Shade", cost(&[b()]), 1, 1, vec![CreatureType::Shade], vec![Keyword::Lifelink])
    }
}

/// Senate Courier — {2}{U} 1/4 Bird with flying. {1}{W}: gains vigilance until EOT.
pub fn senate_courier() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..body("Senate Courier", cost(&[generic(2), u()]), 1, 4, vec![CreatureType::Bird], vec![Keyword::Flying])
    }
}

/// Enraged Ceratok — {2}{G}{G} 4/4 Rhino. Can't be blocked by creatures with
/// power 2 or less.
pub fn enraged_ceratok() -> CardDefinition {
    body("Enraged Ceratok", cost(&[generic(2), g(), g()]), 4, 4, vec![CreatureType::Rhino], vec![Keyword::CantBeBlockedByPowerAtMost(2)])
}

/// Debtors' Transport — {5}{B} 5/3 Thrull with afterlife 2.
pub fn debtors_transport() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(2)],
        ..body("Debtors' Transport", cost(&[generic(5), b()]), 5, 3, vec![CreatureType::Thrull], vec![])
    }
}

/// Spikewheel Acrobat — {3}{R} 5/2 Human Rogue with Spectacle {2}{R}.
pub fn spikewheel_acrobat() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(spectacle(cost(&[generic(2), r()]))),
        ..body("Spikewheel Acrobat", cost(&[generic(3), r()]), 5, 2, vec![CreatureType::Human, CreatureType::Rogue], vec![])
    }
}

/// Dagger Caster — {3}{R} 2/3 Lizard Rogue. ETB deals 1 damage to each opponent
/// and 1 damage to each creature your opponents control.
pub fn dagger_caster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            deal(1, Selector::Player(PlayerRef::EachOpponent)),
            Effect::DealDamage { to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)), amount: Value::Const(1) },
        ]))],
        ..body("Dagger Caster", cost(&[generic(3), r()]), 2, 3, vec![CreatureType::Lizard, CreatureType::Rogue], vec![])
    }
}

/// Footlight Fiend — {B/R} 1/1 Devil. When it dies, deals 1 damage to any target.
pub fn footlight_fiend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::dies_ping_any(1)],
        ..body("Footlight Fiend", cost(&[hybrid(Color::Black, Color::Red)]), 1, 1, vec![CreatureType::Devil], vec![])
    }
}

/// Storm Strike — {R} Instant. Target creature gets +1/+0 and gains first strike
/// until end of turn. Scry 1.
pub fn storm_strike() -> CardDefinition {
    CardDefinition {
        name: "Storm Strike",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::ONE, toughness: Value::Const(0), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Stony Strength — {G} Instant. Put a +1/+1 counter on target creature you
/// control; untap that creature.
pub fn stony_strength() -> CardDefinition {
    CardDefinition {
        name: "Stony Strength",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: target_filtered(R::Creature.and(R::ControlledByYou)), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Ragefire — {1}{R} Sorcery. Deals 3 damage to target creature.
pub fn ragefire() -> CardDefinition {
    CardDefinition {
        name: "Ragefire",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: deal(3, target_filtered(R::Creature)),
        ..Default::default()
    }
}

/// Deface — {R} Sorcery. Choose one — destroy target artifact; or destroy target
/// creature with defender.
pub fn deface() -> CardDefinition {
    CardDefinition {
        name: "Deface",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::Destroy { what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Defender))) },
        ]),
        ..Default::default()
    }
}

/// Elite Arrester — {W} 0/3 Human Soldier. {1}{U}, {T}: Tap target creature.
pub fn elite_arrester() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..body("Elite Arrester", cost(&[w()]), 0, 3, vec![CreatureType::Human, CreatureType::Soldier], vec![])
    }
}

/// Wall of Lost Thoughts — {1}{U} 0/4 Wall with defender. ETB target player mills 4.
pub fn wall_of_lost_thoughts() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(4) })],
        ..body("Wall of Lost Thoughts", cost(&[generic(1), u()]), 0, 4, vec![CreatureType::Wall], vec![Keyword::Defender])
    }
}

/// Thought Collapse — {1}{U}{U} Instant. Counter target spell; its controller mills 3.
pub fn thought_collapse() -> CardDefinition {
    CardDefinition {
        name: "Thought Collapse",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::Any) },
            Effect::Mill { who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))), amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Skatewing Spy — {3}{U} 2/3 Vedalken Rogue Mutant. {5}{U}: Adapt 2. Each
/// creature you control with a +1/+1 counter has flying.
pub fn skatewing_spy() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u()]),
            effect: adapt(2),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter has flying.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::WithCounter(CounterType::PlusOnePlusOne),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Flying],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body("Skatewing Spy", cost(&[generic(3), u()]), 2, 3, vec![CreatureType::Vedalken, CreatureType::Rogue, CreatureType::Mutant], vec![])
    }
}

/// Spirit of the Spires — {3}{W} 2/4 Spirit with flying. Other creatures you
/// control with flying get +0/+1.
pub fn spirit_of_the_spires() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with flying get +0/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasKeyword(Keyword::Flying).and(R::OtherThanSource),
                power: 0,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body("Spirit of the Spires", cost(&[generic(3), w()]), 2, 4, vec![CreatureType::Spirit], vec![Keyword::Flying])
    }
}

/// Shimmer of Possibility — {1}{U} Sorcery. Look at the top four cards of your
/// library. Put one of them into your hand and the rest on the bottom in a
/// random order.
pub fn shimmer_of_possibility() -> CardDefinition {
    CardDefinition {
        name: "Shimmer of Possibility",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: None,
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
            picked_lands_to_battlefield: false,
            rest_bottom_random: true,
        },
        ..Default::default()
    }
}

/// Dead Revels — {3}{B} Sorcery with Spectacle {1}{B}. Return up to two creature
/// cards from your graveyard to your hand.
pub fn dead_revels() -> CardDefinition {
    CardDefinition {
        name: "Dead Revels",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        alternative_cost: Some(spectacle(cost(&[generic(1), b()]))),
        effect: Effect::ReturnGraveyardCardsToHand { filter: R::Creature, max: Value::Const(2) },
        ..Default::default()
    }
}

/// Clamor Shaman — {2}{R} 1/1 Goblin Shaman with riot. Whenever it attacks,
/// target creature an opponent controls can't block this turn.
pub fn clamor_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            riot(),
            on_attack(Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
        ],
        ..body("Clamor Shaman", cost(&[generic(2), r()]), 1, 1, vec![CreatureType::Goblin, CreatureType::Shaman], vec![])
    }
}

/// Resolute Watchdog — {W} 1/3 Dog with defender. {1}, Sacrifice this creature:
/// Target creature you control gains indestructible until end of turn.
pub fn resolute_watchdog() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body("Resolute Watchdog", cost(&[w()]), 1, 3, vec![CreatureType::Dog], vec![Keyword::Defender])
    }
}

/// Tenth District Veteran — {2}{W} 2/3 Human Soldier with vigilance. Whenever it
/// attacks, untap another target creature you control.
pub fn tenth_district_veteran() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Untap {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            up_to: None,
        })],
        ..body("Tenth District Veteran", cost(&[generic(2), w()]), 2, 3, vec![CreatureType::Human, CreatureType::Soldier], vec![Keyword::Vigilance])
    }
}

/// Silhana Wayfinder — {1}{G} 2/1 Elf Scout. ETB look at the top four cards; you
/// may reveal a creature or land from among them and put it on top of your
/// library. Put the rest on the bottom in a random order.
pub fn silhana_wayfinder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: R::Creature.or(R::Land),
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            cap: Value::Const(4),
            life_per_revealed: 0,
            miss_dest: RevealMissDest::BottomRandom,
        })],
        ..body("Silhana Wayfinder", cost(&[generic(1), g()]), 2, 1, vec![CreatureType::Elf, CreatureType::Scout], vec![])
    }
}
