//! Ravnica Allegiance (RNA) — 2019. Commons/uncommons on existing primitives.
//! Tests in `classic_sets/rna`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, MayPlayDuration, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::card::SelectionRequirement as R;
use crate::effect::shortcut::{
    adapt, afterlife, deal, draw, etb, mentor, on_dies, riot, spectacle, target_any,
    target_filtered,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, StaticEffect, ZoneDest,
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

/// Gyre Engineer — {1}{G}{U} 1/1 Vedalken Wizard. {T}: Add {G}{U}. (The adapt
/// payoff rider is engine-supported but omitted here.)
pub fn gyre_engineer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Blue]) },
            ..Default::default()
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

/// Hunted Witness — {W} 1/1 Human. When it dies, create a 1/1 white Soldier
/// with lifelink.
pub fn hunted_witness() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: token("Soldier", vec![Color::White], 1, 1, vec![CreatureType::Soldier], vec![Keyword::Lifelink]),
        })],
        ..body("Hunted Witness", cost(&[w()]), 1, 1, vec![CreatureType::Human], vec![])
    }
}

/// Ministrant of Obligation — {2}{W} 2/1 Human Cleric with Afterlife 2.
pub fn ministrant_of_obligation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(2)],
        ..body("Ministrant of Obligation", cost(&[generic(2), w()]), 2, 1, vec![CreatureType::Human, CreatureType::Cleric], vec![])
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

/// Grasping Thrull — {3}{W}{B} 3/3 Thrull with flying. ETB deal 2 to each
/// opponent and gain 2 life.
pub fn grasping_thrull() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..body("Grasping Thrull", cost(&[generic(3), w(), b()]), 3, 3, vec![CreatureType::Thrull], vec![Keyword::Flying])
    }
}

/// Zhur-Taa Goblin — {R}{G} 2/2 Goblin Berserker with Riot.
pub fn zhur_taa_goblin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        ..body("Zhur-Taa Goblin", cost(&[r(), g()]), 2, 2, vec![CreatureType::Goblin, CreatureType::Berserker], vec![])
    }
}

/// Rampaging Rendhorn — {4}{G} 4/4 Beast with Riot.
pub fn rampaging_rendhorn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        ..body("Rampaging Rendhorn", cost(&[generic(4), g()]), 4, 4, vec![CreatureType::Beast], vec![])
    }
}

/// Frenzied Arynx — {2}{R}{G} 3/3 Cat Beast with Riot and trample. {4}{R}{G}:
/// +3/+0 until end of turn.
pub fn frenzied_arynx() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r(), g()]),
            effect: Effect::PumpPT { what: Selector::This, power: Value::Const(3), toughness: Value::ZERO, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..body("Frenzied Arynx", cost(&[generic(2), r(), g()]), 3, 3, vec![CreatureType::Cat, CreatureType::Beast], vec![Keyword::Trample])
    }
}

/// Sunhome Stalwart — {1}{W} 2/2 Human Soldier with first strike and Mentor.
pub fn sunhome_stalwart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![mentor()],
        ..body("Sunhome Stalwart", cost(&[generic(1), w()]), 2, 2, vec![CreatureType::Human, CreatureType::Soldier], vec![Keyword::FirstStrike])
    }
}

/// Skewer the Critics — {2}{R} Sorcery with Spectacle {R}. Deals 3 damage to
/// any target.
pub fn skewer_the_critics() -> CardDefinition {
    CardDefinition {
        name: "Skewer the Critics",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        alternative_cost: Some(spectacle(cost(&[r()]))),
        effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Light Up the Stage — {2}{R} Sorcery with Spectacle {R}. Exile the top two
/// cards; until the end of your next turn you may play them.
pub fn light_up_the_stage() -> CardDefinition {
    CardDefinition {
        name: "Light Up the Stage",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        alternative_cost: Some(spectacle(cost(&[r()]))),
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(2),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            pay_own_cost: true,
            uncast_penalty: None,
        },
        ..Default::default()
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

/// Gift of Strength — {1}{G} Instant. Target creature gets +3/+3 and gains
/// reach until end of turn.
pub fn gift_of_strength() -> CardDefinition {
    CardDefinition {
        name: "Gift of Strength",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(3), toughness: Value::Const(3), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Reach, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
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

/// Wild Ceratok — {3}{G} 4/3 Rhino.
pub fn wild_ceratok() -> CardDefinition {
    body("Wild Ceratok", cost(&[generic(3), g()]), 4, 3, vec![CreatureType::Rhino], vec![])
}

/// Rubble Slinger — {2}{R/G} 2/3 Human Warrior with reach.
pub fn rubble_slinger() -> CardDefinition {
    body("Rubble Slinger", cost(&[generic(2), hybrid(Color::Red, Color::Green)]), 2, 3, vec![CreatureType::Human, CreatureType::Warrior], vec![Keyword::Reach])
}

/// Impassioned Orator — {1}{W} 2/2 Human Cleric. Whenever another creature you
/// control enters, you gain 1 life.
pub fn impassioned_orator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::OtherThanSource),
            }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..body("Impassioned Orator", cost(&[generic(1), w()]), 2, 2, vec![CreatureType::Human, CreatureType::Cleric], vec![])
    }
}

/// Concordia Pegasus — {1}{W} 1/3 Pegasus with flying.
pub fn concordia_pegasus() -> CardDefinition {
    body("Concordia Pegasus", cost(&[generic(1), w()]), 1, 3, vec![CreatureType::Pegasus], vec![Keyword::Flying])
}

/// Prowling Caracal — {1}{W} 3/1 Cat.
pub fn prowling_caracal() -> CardDefinition {
    body("Prowling Caracal", cost(&[generic(1), w()]), 3, 1, vec![CreatureType::Cat], vec![])
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

/// Bankrupt in Blood — {1}{B} Sorcery. Additional cost: sacrifice two creatures.
/// Draw three cards.
pub fn bankrupt_in_blood() -> CardDefinition {
    CardDefinition {
        name: "Bankrupt in Blood",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 2 }],
        effect: draw(3),
        ..Default::default()
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

/// Territorial Boar — {1}{G} 2/2 Boar. Whenever a creature you control with
/// power 4+ enters, it gets +1/+1 and gains vigilance until end of turn.
pub fn territorial_boar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::PowerAtLeast(4)),
            }),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            ]),
        }],
        ..body("Territorial Boar", cost(&[generic(1), g()]), 2, 2, vec![CreatureType::Boar], vec![])
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

/// Sphinx's Insight — {2}{W}{U} Instant. Draw two cards. Addendum — if cast
/// during your main phase, gain 2 life.
pub fn sphinxs_insight() -> CardDefinition {
    CardDefinition {
        name: "Sphinx's Insight",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            draw(2),
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Bladebrand — {1}{B} Instant. Target creature gains deathtouch until end of
/// turn. Draw a card.
pub fn bladebrand() -> CardDefinition {
    CardDefinition {
        name: "Bladebrand",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            draw(1),
        ]),
        ..Default::default()
    }
}
