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

fn officer_token() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Officer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Officer], ..Default::default() },
        ..Default::default()
    })
}

fn officers(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: officer_token() }
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

// ── Captain Pike, Fullness of Time's EDHREC gaps (scan of 2026-11-27) ───────

/// Captain Pike, Fullness of Time — {1}{G}{G} legendary */* Human Officer
/// Scout; P/T = differently named lands you control. You may look at the top
/// card of your library any time and play lands from it.
pub fn captain_pike_fullness_of_time() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(crate::card::DynamicPt::DistinctLandNamesControlled),
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may play lands from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Land },
            },
        ],
        ..legend(
            "Captain Pike, Fullness of Time",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Human, CreatureType::Officer, CreatureType::Scout],
            0,
            0,
        )
    }
}

/// Awakened Amalgam — {4} */* Golem artifact creature; P/T = differently named
/// lands you control.
pub fn awakened_amalgam() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        dynamic_pt: Some(crate::card::DynamicPt::DistinctLandNamesControlled),
        ..creature("Awakened Amalgam", cost(&[generic(4)]), vec![CreatureType::Golem], 0, 0)
    }
}

/// Branching Timeline — {6}{G}{G} sorcery. Strange new worlds — reveal the top
/// X (X = differently named lands you control); every permanent card enters
/// tapped, the rest go to the bottom in a random order.
pub fn branching_timeline() -> CardDefinition {
    CardDefinition {
        name: "Branching Timeline",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealTopNPutMatchingToBattlefield {
            who: PlayerRef::You,
            count: land_names(),
            filter: R::PermanentCard,
            all_tapped: true,
        },
        ..Default::default()
    }
}

// ── Zero, Incorporeal Sensate's EDHREC gaps (scan of 2026-11-27) ────────────

/// Zero, Incorporeal Sensate — {1}{W/B}{W/B} legendary 2/2 Medusan Scientist
/// artifact creature with lifelink. Entering, look at target opponent's hand
/// and exile a nonland card from it; its owner may play it for {2} more for as
/// long as it remains exiled.
pub fn zero_incorporeal_sensate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::ExileFromHandTaxed {
            from: target_filtered(R::OpponentPlayer),
            count: Value::ONE,
            filter: R::Nonland,
            extra_cost: 2,
        })],
        ..artifact_legend(
            "Zero, Incorporeal Sensate",
            cost(&[generic(1), hybrid(Color::White, Color::Black), hybrid(Color::White, Color::Black)]),
            vec![CreatureType::Medusan, CreatureType::Scientist],
            2,
            2,
        )
    }
}

/// Pick the Brain — {2}{B} sorcery. Target opponent reveals their hand; you
/// exile a nonland card from it. Delirium — also exile every card with that
/// name from their graveyard, hand and library; they shuffle.
pub fn pick_the_brain() -> CardDefinition {
    CardDefinition {
        name: "Pick the Brain",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileChosenFromHand {
                from: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Nonland,
                link_to_source: false,
                face_down: false,
            },
            Effect::If {
                cond: Predicate::DeliriumActive { who: PlayerRef::You },
                then: Box::new(Effect::ExileLibraryCardsNamedLikeExiledThisResolution {
                    who: PlayerRef::Target(0),
                    all_zones: true,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Gul Dukat, Colonial Prefect's EDHREC gaps (scan of 2026-11-27) ──────────

/// "Whenever you face a dilemma" (CR 700.2 — you choose modes for a spell or
/// ability): each opponent loses `n` life and you gain `n`.
fn dilemma_drain(n: i32) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Performed(crate::effect::KeywordAct::FacedDilemma), EventScope::YourControl),
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(n) },
        ]),
    }
}

/// Elim Garak, Simple Tailor — {2}{B} legendary 3/3 Cardassian Rogue with
/// menace. Whenever you face a dilemma, each opponent loses 2 life and you
/// gain 2 life.
pub fn elim_garak_simple_tailor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![dilemma_drain(2)],
        ..legend(
            "Elim Garak, Simple Tailor",
            cost(&[generic(2), b()]),
            vec![CreatureType::Cardassian, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Obsidian Order Operative — {3}{B} 2/5 Cardassian Officer Spy. Whenever you
/// face a dilemma, each opponent loses 1 life and you gain 1 life.
pub fn obsidian_order_operative() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![dilemma_drain(1)],
        ..creature(
            "Obsidian Order Operative",
            cost(&[generic(3), b()]),
            vec![CreatureType::Cardassian, CreatureType::Officer, CreatureType::Spy],
            2,
            5,
        )
    }
}

/// Impossible Choice — {2}{B} instant. Choose one — destroy target creature
/// and its controller creates two 1/1 red Officers; or you draw two cards and
/// each player loses 2 life.
pub fn impossible_choice() -> CardDefinition {
    CardDefinition {
        name: "Impossible Choice",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature) },
                // "Its controller" reads the destroyed creature's LKI (CR 608.2h).
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    count: Value::Const(2),
                    definition: officer_token(),
                },
            ]),
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
            ]),
        ]),
        ..Default::default()
    }
}

// ── The Caretaker's and Jett Reno's EDHREC gaps (scan of 2026-11-27) ────────

/// Jett Reno, Problem Solver — {1}{U} legendary 1/2 Human Artificer. Whenever
/// she becomes tapped, surveil 1 — or draw instead if her power is 4 or
/// greater. Whenever you face a dilemma, put a +1/+1 counter on her.
pub fn jett_reno_problem_solver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(Value::PowerOf(Box::new(Selector::This)), Value::Const(4)),
                    then: Box::new(draw(Value::ONE)),
                    else_: Box::new(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::Performed(crate::effect::KeywordAct::FacedDilemma),
                    EventScope::YourControl,
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..legend(
            "Jett Reno, Problem Solver",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            2,
        )
    }
}

/// Consider the Prime Directive — {2}{U} instant. Choose one — counter target
/// spell unless its controller pays {3}; or tap up to one target creature, put
/// a stun counter on it, and draw a card.
pub fn consider_the_prime_directive() -> CardDefinition {
    CardDefinition {
        name: "Consider the Prime Directive",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
                if_paid: None,
            },
            Effect::Seq(vec![
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: target_filtered(R::Creature) },
                        Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                    ])),
                },
                draw(Value::ONE),
            ]),
        ]),
        ..Default::default()
    }
}

/// Make It So — {3}{U} sorcery. Choose one — untap target permanent you
/// control and target player draws two cards; or create a 1/1 red Officer and
/// draw two cards.
pub fn make_it_so() -> CardDefinition {
    CardDefinition {
        name: "Make It So",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::ControlledByYou), up_to: None },
                Effect::Draw {
                    who: Selector::TargetFiltered { slot: 1, filter: R::Player },
                    amount: Value::Const(2),
                },
            ]),
            Effect::Seq(vec![officers(Value::ONE), draw(Value::Const(2))]),
        ]),
        ..Default::default()
    }
}

/// The Caretaker — {5}{U} legendary 5/6 Elemental with vigilance and ward {2}.
/// At the beginning of each end step, if you faced a dilemma this turn, target
/// opponent exiles cards from the top of their library until they exile a
/// nonland card; you may cast it without paying its mana cost.
pub fn the_caretaker() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                Predicate::ValueAtLeast(Value::DilemmasFacedThisTurn(PlayerRef::You), Value::ONE),
            ),
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::Seq(vec![
                    Effect::ExileTopUntilNonland { who: PlayerRef::Target(0) },
                    Effect::CastExiledFree { what: Selector::ExiledThisResolution { filter: R::Nonland } },
                ])),
            },
        }],
        ..legend("The Caretaker", cost(&[generic(5), u()]), vec![CreatureType::Elemental], 5, 6)
    }
}

/// Federation Probe — {U} 1/1 flying Construct artifact creature. Entering,
/// choose one — target creature gets +1/+1 until end of turn; or surveil 1.
pub fn federation_probe() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        ]))],
        ..creature("Federation Probe", cost(&[u()]), vec![CreatureType::Construct], 1, 1)
    }
}

/// Kobayashi Maru Scenario — {1}{U}{U} enchantment. At the beginning of your
/// upkeep, choose one that hasn't been chosen — return up to one target
/// creature an opponent controls with mana value 3 or less to its owner's
/// hand; draw two; two 1/1 red Officers; or you lose the game.
pub fn kobayashi_maru_scenario() -> CardDefinition {
    CardDefinition {
        name: "Kobayashi Maru Scenario",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::Move {
                            what: target_filtered(
                                R::Creature.and(R::ControlledByOpponent).and(R::ManaValueAtMost(3)),
                            ),
                            to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                        }),
                    },
                    draw(Value::Const(2)),
                    officers(Value::Const(2)),
                    Effect::LoseGame { who: PlayerRef::You },
                ],
            },
        }],
        ..Default::default()
    }
}

// ── Chief Montgomery Scott's EDHREC gaps (scan of 2026-11-27) ───────────────

/// Chief Montgomery Scott — {2}{G}{U} legendary 3/3 Human Artificer. Strange
/// new worlds — {6}, {T}: you may put a creature or Spacecraft card from your
/// hand onto the battlefield; it gets X +1/+1 counters if a creature and X
/// charge counters if a Spacecraft (X = differently named lands you control).
/// The counters are placed as it lands, not as it enters (INCOMPLETE_CARDS).
pub fn chief_montgomery_scott() -> CardDefinition {
    let spacecraft = || R::HasArtifactSubtype(ArtifactSubtype::Spacecraft);
    let counters_if = |filter: R, kind: CounterType| Effect::If {
        cond: Predicate::EntityMatches { what: Selector::LastMoved, filter },
        then: Box::new(Effect::AddCounter { what: Selector::LastMoved, kind, amount: land_names() }),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(6)]),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.or(spacecraft()),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: Some(Box::new(Effect::Seq(vec![
                    counters_if(R::Creature, CounterType::PlusOnePlusOne),
                    counters_if(spacecraft(), CounterType::Charge),
                ]))),
            },
            ..Default::default()
        }],
        ..legend(
            "Chief Montgomery Scott",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Colossal Clone Master — {5}{U} 0/0 Giant. Strange new worlds — it may enter
/// as a copy of any creature on the battlefield, except it isn't legendary and
/// its power and toughness each equal your differently named lands.
pub fn colossal_clone_master() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(crate::card::EntersAsCopy {
            filter: R::Creature,
            non_legendary: true,
            override_dynamic_pt: Some(crate::card::DynamicPt::DistinctLandNamesControlled),
            ..Default::default()
        }),
        ..creature("Colossal Clone Master", cost(&[generic(5), u()]), vec![CreatureType::Giant], 0, 0)
    }
}

// ── Beverly Crusher's and Sulu, Space Swashbuckler's EDHREC gaps ────────────

/// Miles O'Brien, Transporter Ops — {2}{W} legendary 3/3 Human Artificer.
/// Entering, return up to one other target creature or artifact you control
/// to its owner's hand. At the beginning of each end step, if a permanent was
/// returned to your hand from the battlefield this turn, draw a card.
pub fn miles_obrien_transporter_ops() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        R::Creature.or(R::Artifact).and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                    Predicate::ValueAtLeast(Value::ReturnedToHandThisTurn(PlayerRef::You), Value::ONE),
                ),
                effect: draw(Value::ONE),
            },
        ],
        ..legend(
            "Miles O'Brien, Transporter Ops",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Sulu, Space Swashbuckler — {1}{W} legendary 2/2 Human Pilot, first strike.
/// Whenever he becomes tapped, if an Equipment or Spacecraft entered under
/// your control this turn, draw a card (read off those still on the
/// battlefield — INCOMPLETE_CARDS).
pub fn sulu_space_swashbuckler() -> CardDefinition {
    let entered = R::HasArtifactSubtype(ArtifactSubtype::Equipment)
        .or(R::HasArtifactSubtype(ArtifactSubtype::Spacecraft))
        .and(R::ControlledByYou)
        .and(R::EnteredThisTurn);
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource).with_filter(
                Predicate::SelectorCountAtLeast { sel: Selector::EachPermanent(entered), n: Value::ONE },
            ),
            effect: draw(Value::ONE),
        }],
        ..legend("Sulu, Space Swashbuckler", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Pilot], 2, 2)
    }
}

/// Vexing Drone — {1}{W} 2/2 Borg artifact creature. Entering, tap target
/// creature an opponent controls.
pub fn vexing_drone() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Tap {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..creature("Vexing Drone", cost(&[generic(1), w()]), vec![CreatureType::Borg], 2, 2)
    }
}

/// U.S.S. Excelsior, Cutting Edge — {3}{W} legendary Spacecraft. Entering,
/// destroy target tapped creature an opponent controls. Station; 8+: flying
/// 4/4.
pub fn uss_excelsior_cutting_edge() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::Creature.and(R::Tapped).and(R::ControlledByOpponent)),
        })],
        ..spacecraft(
            "U.S.S. Excelsior, Cutting Edge",
            cost(&[generic(3), w()]),
            vec![StationBand { min: 8, keywords: vec![Keyword::Flying], pt: Some((4, 4)), ..Default::default() }],
        )
    }
}

// ── Subcommander T'Pol's EDHREC gaps (scan of 2026-11-27) ───────────────────

/// Tricorder — {1} Equipment, equip {1}. +1/+1 and "whenever this creature
/// attacks, surveil 1".
pub fn tricorder() -> CardDefinition {
    CardDefinition {
        name: "Tricorder",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_attack(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Cryogenic Stasis — {1}{U} instant. Tap target creature, put a stun counter
/// on it, draw a card.
pub fn cryogenic_stasis() -> CardDefinition {
    CardDefinition {
        name: "Cryogenic Stasis",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
            draw(Value::ONE),
        ]),
        ..Default::default()
    }
}

/// Those Old Scientists — {2}{U} Saga. I — draw two, then discard two. II —
/// return up to one target artifact, instant or sorcery card from your
/// graveyard to your hand. III — up to two target cards in graveyards are
/// shuffled into their owners' libraries.
pub fn those_old_scientists() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    use crate::effect::{LibraryPosition, ZoneDest};
    CardDefinition {
        name: "Those Old Scientists",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    draw(Value::Const(2)),
                    Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
                ]),
            ),
            (
                2,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Move {
                        what: target_filtered(
                            R::Artifact.or(R::HasCardType(CardType::Instant)).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard),
                        ),
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                },
            ),
            (
                3,
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::InGraveyard,
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Shuffled },
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}
