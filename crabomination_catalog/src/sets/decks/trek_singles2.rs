//! Star Trek (TRK/TRC) and friends — the second batch of EDHREC Star Trek
//! closers: decks one or two cards short after `trek_singles`. Tests in
//! `tests/recent_b/trek_singles2.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StationBand, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, investigate, on_attack, on_dies, station, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, StaticEffect};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w, x};

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

fn artifact_legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    let mut c = legend(name, mana, types, p, t);
    c.card_types.insert(0, CardType::Artifact);
    c
}

fn spacecraft(name: &'static str, mana: ManaCost, bands: Vec<StationBand>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: bands,
        ..Default::default()
    }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn counter_on(what: Selector, kind: CounterType, n: Value) -> Effect {
    Effect::AddCounter { what, kind, amount: n }
}

fn landfall(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
        effect,
    }
}

fn your_combat(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl), effect }
}

fn your_first_main(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl), effect }
}

/// "The number of differently named lands you control" (strange new worlds).
fn land_names() -> Value {
    Value::DistinctNamesControlledMatching(R::Land)
}

/// "The number of creature types among non-Borg creatures you control"
/// (the Federation ability word).
fn federation() -> Value {
    Value::DistinctCreatureTypesAmongYourNonBorgCreatures
}

fn officers(count: Value) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count,
        definition: Arc::new(TokenDefinition {
            name: "Officer".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            colors: vec![Color::Red],
            subtypes: Subtypes { creature_types: vec![CreatureType::Officer], ..Default::default() },
            ..Default::default()
        }),
    }
}

fn sac_at_end_step(make: Effect) -> Effect {
    Effect::Seq(vec![make, Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedTokens }])
}

// ── Spacecraft ───────────────────────────────────────────────────────────────

/// D-7 Battle Cruiser — {1}{R} Spacecraft. Station; 3+: at the beginning of
/// combat on your turn, a hasty 2/1 Klingon Warrior sacrificed at the next end
/// step; 8+: 3/3 flying.
pub fn d_7_battle_cruiser() -> CardDefinition {
    let warrior = Arc::new(TokenDefinition {
        name: "Klingon Warrior".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Klingon, CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    });
    spacecraft(
        "D-7 Battle Cruiser",
        cost(&[generic(1), r()]),
        vec![
            StationBand {
                min: 3,
                triggers: vec![your_combat(sac_at_end_step(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: warrior,
                }))],
                ..Default::default()
            },
            StationBand { min: 8, keywords: vec![Keyword::Flying], pt: Some((3, 3)), ..Default::default() },
        ],
    )
}

/// Class-F Shuttlecraft — {3} Spacecraft. {T}: one mana of any color.
/// Station; 6+: 2/4 flying, vigilance.
pub fn class_f_shuttlecraft() -> CardDefinition {
    let mut c = spacecraft(
        "Class-F Shuttlecraft",
        cost(&[generic(3)]),
        vec![StationBand {
            min: 6,
            keywords: vec![Keyword::Flying, Keyword::Vigilance],
            pt: Some((2, 4)),
            ..Default::default()
        }],
    );
    c.activated_abilities.insert(0, crate::sets::tap_add_any_color());
    c
}

// ── counters ─────────────────────────────────────────────────────────────────

/// Dr. Joseph M'Benga — {2}{G}{G} legendary 3/3 Human Doctor. Your other
/// creatures with counters have hexproof; counters put on your creatures are
/// doubled.
pub fn dr_joseph_mbenga() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other creatures you control with counters on them have hexproof.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource).and(R::WithAnyCounter),
                    ),
                    keyword: Keyword::Hexproof,
                },
            },
            StaticAbility {
                description: "If you would put one or more counters on a creature you control, put twice that many of each of those kinds of counters on that creature instead.",
                effect: StaticEffect::DoubleCountersMatching { filter: R::Creature.and(R::ControlledByYou) },
            },
        ],
        ..legend("Dr. Joseph M'Benga", cost(&[generic(2), g(), g()]), vec![CreatureType::Human, CreatureType::Doctor], 3, 3)
    }
}

/// Lt. Cmdr. Geordi La Forge — {2}{U} legendary 3/2 Human Artificer. Charge
/// counters put on your artifacts are doubled.
pub fn lt_cmdr_geordi_la_forge() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would put one or more charge counters on an artifact you control, put twice that many charge counters on that artifact instead.",
            effect: StaticEffect::DoubleCountersOfKindMatching {
                kind: CounterType::Charge,
                filter: R::Artifact.and(R::ControlledByYou),
            },
        }],
        ..legend(
            "Lt. Cmdr. Geordi La Forge",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            2,
        )
    }
}

/// Jadzia Dax, Living Legacy — {1}{G} legendary 0/1 Trill Scientist. At your
/// first main phase, a +1/+1 counter, then {G} per counter on her. Transfer
/// Symbiont — {2}{G}, discard this card: two +1/+1 counters on target
/// creature.
pub fn jadzia_dax_living_legacy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_first_main(Effect::Seq(vec![
            counter_on(Selector::This, CounterType::PlusOnePlusOne, Value::ONE),
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                }),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: counter_on(target_filtered(R::Creature), CounterType::PlusOnePlusOne, Value::Const(2)),
            ..Default::default()
        }],
        ..legend("Jadzia Dax, Living Legacy", cost(&[generic(1), g()]), vec![CreatureType::Trill, CreatureType::Scientist], 0, 1)
    }
}

/// Data, New Life — {1}{U} legendary 0/1 Android Scientist artifact creature.
/// Drawing a card puts a +1/+1 counter on him.
pub fn data_new_life() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: counter_on(Selector::This, CounterType::PlusOnePlusOne, Value::ONE),
        }],
        ..artifact_legend("Data, New Life", cost(&[generic(1), u()]), vec![CreatureType::Android, CreatureType::Scientist], 0, 1)
    }
}

/// Lieutenant Commander Data — {2}{U} legendary 2/2 Android Scientist artifact
/// creature. Any player's noncreature spell grows him. {1}, remove three
/// counters from among your creatures: draw a card.
pub fn lieutenant_commander_data() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::Creature.negate())),
            effect: counter_on(Selector::This, CounterType::PlusOnePlusOne, Value::ONE),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_among_filter: Some((None, 3, R::Creature)),
            effect: draw(Value::ONE),
            ..Default::default()
        }],
        ..artifact_legend(
            "Lieutenant Commander Data",
            cost(&[generic(2), u()]),
            vec![CreatureType::Android, CreatureType::Scientist],
            2,
            2,
        )
    }
}

/// Seven of Nine, Unassimilated — {1}{U} legendary 0/4 Human Officer artifact
/// creature. At your first main phase, draw, then discard unless you control
/// five differently named lands. {6}{U}: base 7/9, with seven of them.
pub fn seven_of_nine_unassimilated() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_first_main(Effect::Seq(vec![
            draw(Value::ONE),
            Effect::If {
                cond: Predicate::ValueAtLeast(land_names(), Value::Const(5)),
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), u()]),
            condition: Some(Predicate::ValueAtLeast(land_names(), Value::Const(7))),
            effect: Effect::SetBasePT {
                what: Selector::This,
                power: Value::Const(7),
                toughness: Value::Const(9),
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..artifact_legend(
            "Seven of Nine, Unassimilated",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Officer],
            0,
            4,
        )
    }
}

/// Spock, Logical Choice — {3}{U}{U} legendary 3/5 Vulcan Human Scientist,
/// vigilance. {T}: draw a card.
pub fn spock_logical_choice() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility { tap_cost: true, effect: draw(Value::ONE), ..Default::default() }],
        ..legend(
            "Spock, Logical Choice",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Vulcan, CreatureType::Human, CreatureType::Scientist],
            3,
            5,
        )
    }
}

/// Spock, Always Your Friend — {U}{R} legendary 2/2 Vulcan Human Scientist.
/// Entering, look at the top four; a noncreature, nonland card may go to hand.
pub fn spock_always_your_friend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(R::Creature.negate().and(R::Nonland)),
            optional: true,
            ..Default::default()
        })))],
        ..legend(
            "Spock, Always Your Friend",
            cost(&[u(), r()]),
            vec![CreatureType::Vulcan, CreatureType::Human, CreatureType::Scientist],
            2,
            2,
        )
    }
}

// ── lands ────────────────────────────────────────────────────────────────────

/// The Outpost / Bar cycle: enters tapped unless you control two other lands;
/// entering untapped picks one of two riders.
fn outpost(name: &'static str, color: Color, first: Effect) -> CardDefinition {
    let three_lands = || Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
        n: Value::Const(3),
    };
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control two or more other lands.",
            effect: StaticEffect::EntersTappedUnless { applies_to: Selector::This, condition: three_lands() },
        }],
        activated_abilities: vec![crate::sets::tap_add(color)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Untapped }),
            effect: Effect::ChooseMode(vec![first, Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }]),
        }],
        ..Default::default()
    }
}

/// Federation Outpost — land, {T}: {W}. Entering untapped: gain 2 life or
/// surveil 1.
pub fn federation_outpost() -> CardDefinition {
    outpost("Federation Outpost", Color::White, Effect::GainLife { who: Selector::You, amount: Value::Const(2) })
}

/// Quark's Bar — land, {T}: {R}. Entering untapped: 1 damage to each opponent
/// or surveil 1.
pub fn quarks_bar() -> CardDefinition {
    outpost(
        "Quark's Bar",
        Color::Red,
        Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
    )
}

/// Starfleet Academy — legendary land. {T}: {C}. Federation — {5}, {T}: one
/// mana of a color per creature type among your non-Borg creatures.
pub fn starfleet_academy() -> CardDefinition {
    CardDefinition {
        name: "Starfleet Academy",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(federation()) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Federation ───────────────────────────────────────────────────────────────

/// Captain Jonathan Archer — {2}{G} legendary 3/3 Human Officer. Entering
/// makes Porthos, a legendary 1/1 green Dog. Federation — {G}, {T}: target
/// creature you control gets +X/+X until end of turn; sorcery speed.
pub fn captain_jonathan_archer() -> CardDefinition {
    let porthos = Arc::new(TokenDefinition {
        name: "Porthos".into(),
        power: 1,
        toughness: 1,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: porthos })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: federation(),
                toughness: federation(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend("Captain Jonathan Archer", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Officer], 3, 3)
    }
}

/// Sarek, Federation Ambassador — {1}{G}{W}{U} legendary 0/5 Vulcan Advisor.
/// Federation — at the beginning of combat on your turn, draw a card per five
/// creature types; three or more cards drawn put two +1/+1 counters on each of
/// your creatures.
pub fn sarek_federation_ambassador() -> CardDefinition {
    let n = || Value::DivDown(Box::new(federation()), 5);
    CardDefinition {
        triggered_abilities: vec![your_combat(Effect::Seq(vec![
            draw(n()),
            Effect::If {
                cond: Predicate::ValueAtLeast(n(), Value::Const(3)),
                then: Box::new(counter_on(
                    Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    CounterType::PlusOnePlusOne,
                    Value::Const(2),
                )),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..legend(
            "Sarek, Federation Ambassador",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Vulcan, CreatureType::Advisor],
            0,
            5,
        )
    }
}

// ── the rest ─────────────────────────────────────────────────────────────────

/// Angelheart Vial — {5} artifact. Being dealt damage, you may put that many
/// charge counters on it. {2}, {T}, remove four charge counters: gain 2 life
/// and draw a card.
pub fn angelheart_vial() -> CardDefinition {
    CardDefinition {
        name: "Angelheart Vial",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::SourceDamagedYou),
            effect: Effect::MayDo {
                description: "Put that many charge counters on Angelheart Vial?".into(),
                body: Box::new(counter_on(Selector::This, CounterType::Charge, Value::TriggerEventAmount)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Charge, 4)),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                draw(Value::ONE),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Captain of the Mists — {2}{U} 2/3 Human Wizard. Another Human of yours
/// entering untaps it. {1}{U}, {T}: tap or untap target permanent.
pub fn captain_of_the_mists() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Human) },
            ),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::TapOrUntap { what: target_filtered(R::Permanent) },
            ..Default::default()
        }],
        ..creature("Captain of the Mists", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

/// Jake Sisko, War Correspondent — {1}{G} legendary 2/2 Human Citizen.
/// Landfall: gain 1 life; the second resolution each turn also investigates.
pub fn jake_sisko_war_correspondent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
            Effect::NthResolutionThisTurn { branches: vec![Effect::Noop, investigate(1)] },
        ]))],
        ..legend(
            "Jake Sisko, War Correspondent",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            2,
            2,
        )
    }
}

/// Borrowing the East Wind — {X}{G}{G} sorcery. X damage to each creature
/// with horsemanship and each player.
pub fn borrowing_the_east_wind() -> CardDefinition {
    CardDefinition {
        name: "Borrowing the East Wind",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Horsemanship))),
                amount: Value::XFromCost,
            },
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::XFromCost },
        ]),
        ..Default::default()
    }
}

/// Excalbian Gamemaster — {5}{W} 6/6 Excalbian. Strange new worlds — entering
/// makes a 1/1 red Officer per differently named land you control. It or
/// another creature of yours dying puts a +1/+1 counter on target creature
/// you control.
pub fn excalbian_gamemaster() -> CardDefinition {
    let grow = || counter_on(target_filtered(R::Creature.and(R::ControlledByYou)), CounterType::PlusOnePlusOne, Value::ONE);
    CardDefinition {
        triggered_abilities: vec![
            etb(officers(land_names())),
            on_dies(grow()),
            TriggeredAbility { event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours), effect: grow() },
        ],
        ..creature("Excalbian Gamemaster", cost(&[generic(5), w()]), vec![CreatureType::Excalbian], 6, 6)
    }
}

/// Kithkin Spellduster — {4}{W} 2/3 Kithkin Wizard, flying, persist. {1}{W},
/// sacrifice it: destroy target enchantment.
pub fn kithkin_spellduster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Persist],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
            ..Default::default()
        }],
        ..creature("Kithkin Spellduster", cost(&[generic(4), w()]), vec![CreatureType::Kithkin, CreatureType::Wizard], 2, 3)
    }
}

/// Dal R'El, Augment Orphan — {1}{R}{G} legendary 3/3 Human Mutant Rogue.
/// Landfall: add {R} or {G}. {3}, sacrifice a land: draw a card.
pub fn dal_rel_augment_orphan() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColors(vec![Color::Red, Color::Green], Value::ONE),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_other_filter: Some((R::Land, 1)),
            effect: draw(Value::ONE),
            ..Default::default()
        }],
        ..legend(
            "Dal R'El, Augment Orphan",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Human, CreatureType::Mutant, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Exploratory Mission — {2}{R}{R} enchantment. Landfall: a quest counter,
/// then exile down to a nonland card you may play this turn; at five or more
/// quest counters, add {R}{R}.
pub fn exploratory_mission() -> CardDefinition {
    CardDefinition {
        name: "Exploratory Mission",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![landfall(Effect::Seq(vec![
            counter_on(Selector::This, CounterType::Quest, Value::ONE),
            Effect::ExileTopUntilNonlandMayPlay {
                who: PlayerRef::You,
                duration: MayPlayDuration::EndOfThisTurn,
                free: false,
                hand_unless_mv_below: None,
                grant_to_exiling_player: false,
            },
            Effect::If {
                cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Quest, n: 5 },
                then: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Red, Color::Red]),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Gray and Adira, Forever Joined — {2}{U/R}{U/R} legendary 2/3 Trill Human
/// Officer. At the beginning of combat on your turn, if you've cast a
/// noncreature spell this turn, a hasty nonlegendary token copy of target
/// creature you control, sacrificed at the next end step.
pub fn gray_and_adira_forever_joined() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(Value::NoncreatureSpellsCastThisTurn(PlayerRef::You), Value::ONE),
            ),
            effect: sac_at_end_step(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::Creature.and(R::ControlledByYou)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: true,
                legendary: false,
                extra_keywords: vec![Keyword::Haste],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            }),
        }],
        ..legend(
            "Gray and Adira, Forever Joined",
            cost(&[generic(2), ur(), ur()]),
            vec![CreatureType::Trill, CreatureType::Human, CreatureType::Officer],
            2,
            3,
        )
    }
}

/// Khan, Vengeful Heart — {2}{B}{B} legendary 5/3 Human Warrior. Entering,
/// you may sacrifice another creature or artifact to draw two. You may cast
/// this card from exile.
pub fn khan_vengeful_heart() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ExileCast],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Sacrifice another creature or artifact to draw two cards?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature.or(R::Artifact).and(R::OtherThanSource),
                },
                Effect::If {
                    cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                    then: Box::new(draw(Value::Const(2))),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..legend("Khan, Vengeful Heart", cost(&[generic(2), b(), b()]), vec![CreatureType::Human, CreatureType::Warrior], 5, 3)
    }
}

/// Sela, Tal Shiar Paradox — {3}{B} legendary 3/3 Romulan Human Rogue.
/// Attacking exiles up to one target card from a graveyard. One or more cards
/// leaving your graveyard make a tapped 2/2 black Romulan.
pub fn sela_tal_shiar_paradox() -> CardDefinition {
    let romulan = Arc::new(TokenDefinition {
        name: "Romulan".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Romulan], ..Default::default() },
        tapped: true,
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Exile { what: target_filtered(R::Any.from_any_graveyard()) }),
            }),
            TriggeredAbility {
                event: EventSpec { once_per_batch: true, ..EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl) },
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: romulan },
            },
        ],
        ..legend(
            "Sela, Tal Shiar Paradox",
            cost(&[generic(3), b()]),
            vec![CreatureType::Romulan, CreatureType::Human, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Mega Flare — {2}{R} sorcery, kicker {3}{R}{R}. Kicked, a 6/6 red flying
/// Dragon first. For each opponent, up to one target creature they control is
/// dealt damage equal to the greatest power among your creatures.
pub fn mega_flare() -> CardDefinition {
    let dragon = Arc::new(TokenDefinition {
        name: "Dragon".into(),
        power: 6,
        toughness: 6,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    });
    CardDefinition {
        name: "Mega Flare",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(3), r(), r()]))],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: dragon }),
                else_: Box::new(Effect::Noop),
            },
            Effect::ForEachOpponentTarget {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 15,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByOpponent),
                    effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)) }),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Quark, Humble Proprietor — {2}{R} legendary 1/2 Ferengi Rogue. You can't
/// sacrifice Treasure tokens (they can't be sacrificed while you control
/// them, CR 701.21). At the beginning of each upkeep, a Treasure.
pub fn quark_humble_proprietor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You can't sacrifice Treasure tokens.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasArtifactSubtype(ArtifactSubtype::Treasure).and(R::IsToken).and(R::ControlledByYou),
                ),
                keyword: Keyword::CantBeSacrificed,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: crate::effect::shortcut::mint_treasures(1),
        }],
        ..legend("Quark, Humble Proprietor", cost(&[generic(2), r()]), vec![CreatureType::Ferengi, CreatureType::Rogue], 1, 2)
    }
}
