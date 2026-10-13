//! Commander: gaps of EDHREC's commanders ranked 1501-2000 (the 2026-10-13
//! twenty-page rescan), fourth file. Tests in `tests/recent_b/cmdr_rank2k_d.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, landfall, on_attack, target_any, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, u, w, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn grant_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(def) }
}

fn tempt() -> Effect {
    Effect::RingTempts { who: PlayerRef::You }
}

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

/// Archwing Dragon — {2}{R}{R} 4/4 flying, haste Dragon; returns to its
/// owner's hand at the beginning of each end step.
pub fn archwing_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
        }],
        ..creature("Archwing Dragon", cost(&[generic(2), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Arwen, Mortal Queen — {1}{G}{W} legendary 2/2 Elf Noble; enters with an
/// indestructible counter. {1}, remove it: another creature gains
/// indestructible until end of turn, and it and Arwen each get a +1/+1 and a
/// lifelink counter.
pub fn arwen_mortal_queen() -> CardDefinition {
    let marks = |what: Selector| {
        vec![
            Effect::AddCounter { what: what.clone(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::AddKeywordCounter { what, keyword: Keyword::Lifelink, amount: Value::ONE },
        ]
    };
    let mut body = vec![grant_eot(target_filtered(R::Creature.and(R::OtherThanSource)), Keyword::Indestructible)];
    body.extend(marks(Selector::Target(0)));
    body.extend(marks(Selector::This));
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddKeywordCounter {
            what: Selector::This,
            keyword: Keyword::Indestructible,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_keyword_counter_cost: Some(Keyword::Indestructible),
            effect: Effect::Seq(body),
            ..Default::default()
        }],
        ..legend("Arwen, Mortal Queen", cost(&[generic(1), g(), w()]), vec![CreatureType::Elf, CreatureType::Noble], 2, 2)
    }
}

/// Daughter of the Deep — {1}{U} 2/1 Merfolk Noble. Your second draw each
/// turn makes a 1/1 blue Merfolk; {U}, {T}: a Merfolk can't be blocked.
pub fn daughter_of_the_deep() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: make(Value::ONE, token("Merfolk", 1, 1, Color::Blue, vec![CreatureType::Merfolk])),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: grant_eot(target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Merfolk))), Keyword::Unblockable),
            ..Default::default()
        }],
        ..creature("Daughter of the Deep", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Noble], 2, 1)
    }
}

/// Elrond, Lord of Rivendell — {2}{U} legendary 3/2 Elf Noble. It or another
/// creature of yours entering scries 1; the second resolution each turn also
/// has the Ring tempt you.
pub fn elrond_lord_of_rivendell() -> CardDefinition {
    let scry = || Effect::Scry { who: PlayerRef::You, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::EscalatingThisTurn { modes: vec![scry(), Effect::Seq(vec![scry(), tempt()]), scry()] },
        }],
        ..legend("Elrond, Lord of Rivendell", cost(&[generic(2), u()]), vec![CreatureType::Elf, CreatureType::Noble], 3, 2)
    }
}

/// Éowyn, Lady of Rohan — {2}{W} legendary 2/4 Human Noble. Beginning of
/// combat on your turn: a creature gains first strike or vigilance — both if
/// it's equipped. Your equip abilities cost {1} less.
pub fn eowyn_lady_of_rohan() -> CardDefinition {
    let mode = |first: Keyword, second: Keyword| {
        Effect::Seq(vec![
            grant_eot(target_filtered(R::Creature), first),
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::IsEquipped },
                then: Box::new(grant_eot(Selector::Target(0), second)),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![at_your(
            TurnStep::BeginCombat,
            Effect::ChooseMode(vec![
                mode(Keyword::FirstStrike, Keyword::Vigilance),
                mode(Keyword::Vigilance, Keyword::FirstStrike),
            ]),
        )],
        static_abilities: vec![static_ab(
            "Equip abilities you activate cost {1} less to activate.",
            StaticEffect::EquipCostReduction { amount: 1 },
        )],
        ..legend("Éowyn, Lady of Rohan", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Noble], 2, 4)
    }
}

/// Fortune's Favor — {3}{U} instant: an opponent splits your top four into a
/// face-down and a face-up pile; one pile to hand, the other to the
/// graveyard. (The split rides the Fact or Fiction resolver.)
pub fn fortunes_favor() -> CardDefinition {
    spell(
        "Fortune's Favor",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::FactOrFiction { count: Value::Const(4), to_bottom: false },
    )
}

/// Galadriel, Gift-Giver — {3}{G}{G} legendary 4/4 Elf Noble. Entering or
/// attacking: a +1/+1 counter on another creature, a Food, or a Treasure.
pub fn galadriel_gift_giver() -> CardDefinition {
    let modal = || {
        Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            crate::effect::shortcut::mint_token(crabomination_base::tokens::food_token(), 1),
            crate::effect::shortcut::mint_treasures(1),
        ])
    };
    CardDefinition {
        triggered_abilities: vec![etb(modal()), on_attack(modal())],
        ..legend("Galadriel, Gift-Giver", cost(&[generic(3), g(), g()]), vec![CreatureType::Elf, CreatureType::Noble], 4, 4)
    }
}

/// Glowspore Shaman — {B}{G} 3/1 Elf Shaman. Entering mills three; you may
/// put a land card from your graveyard on top of your library.
pub fn glowspore_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::MayDo {
                description: "Put a land card from your graveyard on top of your library?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Land.and(R::InYourGraveyard)),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                }),
            },
        ]))],
        ..creature(
            "Glowspore Shaman",
            cost(&[crate::mana::b(), g()]),
            vec![CreatureType::Elf, CreatureType::Shaman],
            3,
            1,
        )
    }
}

/// Go Ninja Go — {R}{W} sorcery, choose one or both: blink a creature you
/// control; deal damage equal to your greatest power to an opponent's creature.
pub fn go_ninja_go() -> CardDefinition {
    spell(
        "Go Ninja Go",
        cost(&[r(), w()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            min: 1,
            max: 2,
            allow_repeats: false,
            modes: vec![
                Effect::Seq(vec![
                    Effect::Exile { what: target_filtered(yours(R::Creature)) },
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                    },
                ]),
                Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    amount: Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(R::Creature))),
                },
            ],
        },
    )
}

/// Immard, the Stormcleaver — {1}{U}{R}{W} legendary 4/4 Human Soldier.
/// Entering or attacking, put a charge counter on it or remove one; removing
/// one deals 4 damage to any target or grants lifelink and indestructible.
pub fn immard_the_stormcleaver() -> CardDefinition {
    let charge = || {
        Effect::ChooseMode(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
            Effect::If {
                cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Charge, n: 1 },
                then: Box::new(Effect::Seq(vec![
                    Effect::RemoveCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
                    Effect::ChooseMode(vec![
                        Effect::DealDamage { to: target_any(), amount: Value::Const(4) },
                        Effect::GrantKeywords {
                            what: Selector::This,
                            keywords: vec![Keyword::Lifelink, Keyword::Indestructible],
                            duration: Duration::EndOfTurn,
                        },
                    ]),
                ])),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![etb(charge()), on_attack(charge())],
        ..legend(
            "Immard, the Stormcleaver",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Jolrael, Voice of Zhalfir — {2}{G}{U} legendary 3/3 Human Druid. Beginning
/// of combat on your turn, up to one land you control becomes an X/X green and
/// blue flying, hasty Bird (X = your hand size) until end of turn. A land
/// creature of yours dealing combat damage to a player draws a card.
pub fn jolrael_voice_of_zhalfir() -> CardDefinition {
    let x = || Value::HandSizeOf(PlayerRef::You);
    CardDefinition {
        triggered_abilities: vec![
            at_your(
                TurnStep::BeginCombat,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(crate::effect::shortcut::colored_animation(
                        Effect::BecomeCreature {
                            what: target_filtered(yours(R::Land)),
                            power: x(),
                            toughness: x(),
                            creature_types: vec![CreatureType::Bird],
                            keywords: vec![Keyword::Flying, Keyword::Haste],
                            duration: Duration::EndOfTurn,
                        },
                        &[Color::Green, Color::Blue],
                    )),
                },
            ),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land.and(R::Creature) },
                ),
                effect: draw(1),
            },
        ],
        ..legend(
            "Jolrael, Voice of Zhalfir",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Human, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Metropolis Angel — {2}{W}{U} 3/1 flying Angel Soldier; attacking with a
/// creature that has counters draws a card.
pub fn metropolis_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: R::WithAnyCounter }),
            effect: draw(1),
        }],
        ..creature(
            "Metropolis Angel",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Angel, CreatureType::Soldier],
            3,
            1,
        )
    }
}

/// Nahiri's Machinations — {1}{W} enchantment. Beginning of combat on your
/// turn, a creature you control gains indestructible; {1}{R}: 1 damage to a
/// blocking creature.
pub fn nahiris_machinations() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![at_your(
            TurnStep::BeginCombat,
            grant_eot(target_filtered(yours(R::Creature)), Keyword::Indestructible),
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::DealDamage { to: target_filtered(R::Creature.and(R::IsBlocking)), amount: Value::ONE },
            ..Default::default()
        }],
        ..spell("Nahiri's Machinations", cost(&[generic(1), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Rally for the Throne — {2}{W} instant: two 1/1 white Humans; adamant
/// (three white spent): gain a life per creature you control.
pub fn rally_for_the_throne() -> CardDefinition {
    spell(
        "Rally for the Throne",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            make(Value::Const(2), token("Human", 1, 1, Color::White, vec![CreatureType::Human])),
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast { color: Color::White, at_least: 3 },
                then: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(yours(R::Creature)))),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Rangers of Ithilien — {2}{U}{U} 3/3 vigilance Human Ranger. Entering, take
/// up to one creature with lesser power for as long as you control it; then
/// the Ring tempts you.
pub fn rangers_of_ithilien() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::GainControlWhileYouControlSource {
                    what: target_filtered(R::Creature.and(R::PowerLessThanSource)),
                }),
            },
            tempt(),
        ]))],
        ..creature("Rangers of Ithilien", cost(&[generic(2), u(), u()]), vec![CreatureType::Human, CreatureType::Ranger], 3, 3)
    }
}

/// Repel the Abominable — {1}{W} instant: prevent all damage non-Human
/// sources would deal this turn.
pub fn repel_the_abominable() -> CardDefinition {
    spell(
        "Repel the Abominable",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::PreventAllDamageByMatchingThisTurn { filter: R::HasCreatureType(CreatureType::Human).negate() },
    )
}

/// Return of the Mole Man — {1}{G} enchantment. Landfall: you may mill two.
/// {5}{G}, sacrifice it (sorcery speed): X 1/1 green Minion tokens named
/// Moloid, X = permanent cards in your graveyard, each with "whenever this
/// attacks, you may mill a card."
pub fn return_of_the_mole_man() -> CardDefinition {
    let mill = |n: i32| Effect::MayDo {
        description: "Mill?".into(),
        body: Box::new(Effect::Mill { who: Selector::You, amount: Value::Const(n) }),
    };
    let moloid = TokenDefinition {
        triggered_abilities: vec![on_attack(mill(1))],
        ..token("Moloid", 1, 1, Color::Green, vec![CreatureType::Minion])
    };
    CardDefinition {
        triggered_abilities: vec![landfall(mill(2))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: make(Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::PermanentCard }, moloid),
            ..Default::default()
        }],
        ..spell("Return of the Mole Man", cost(&[generic(1), g()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Shatterskull Charger — {1}{R}{R} 4/3 trample, haste Giant Warrior, kicker
/// {2} for a +1/+1 counter; returns to hand at your end step unless it has a
/// +1/+1 counter.
pub fn shatterskull_charger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Kicker(cost(&[generic(2)]))],
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                    Predicate::Not(Box::new(Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::PlusOnePlusOne,
                        n: 1,
                    })),
                ),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            },
        ],
        ..creature(
            "Shatterskull Charger",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            4,
            3,
        )
    }
}

/// Spirit en-Dal — {2}{W} 2/1 shadow Spirit. Forecast — {1}{W}, reveal it
/// (your upkeep, once a turn): a creature gains shadow until end of turn.
pub fn spirit_en_dal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        activated_abilities: vec![crate::effect::shortcut::forecast(
            cost(&[generic(1), w()]),
            grant_eot(target_filtered(R::Creature), Keyword::Shadow),
        )],
        ..creature("Spirit en-Dal", cost(&[generic(2), w()]), vec![CreatureType::Spirit], 2, 1)
    }
}

/// Team Transmitter — {3} artifact: a Hero of yours entering gains 1 life;
/// {T}: one mana of any color.
pub fn team_transmitter() -> CardDefinition {
    CardDefinition {
        name: "Team Transmitter",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Hero)),
                },
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..Default::default()
    }
}

/// Templar Knight — {1}{W} 3/1 vigilance Human Knight. {W}, tap five untapped
/// attacking Templar Knights you control: put a legendary artifact card from
/// your library onto the battlefield. Any number in a deck.
pub fn templar_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![crate::sets::deck_may_have_copies(None)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_n_filter: Some((R::HasName("Templar Knight".into()).and(R::IsAttacking), 5)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Artifact.and(R::HasSupertype(Supertype::Legendary)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Templar Knight", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Knight], 3, 1)
    }
}

/// Viconia, Drow Apostate — {2}{B} legendary 2/3 Elf Cleric. Each upkeep with
/// four or more creature cards in your graveyard, one of them at random
/// returns to your hand. Choose a Background.
pub fn viconia_drow_apostate() -> CardDefinition {
    let creatures = || Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature };
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::SelectorCountAtLeast { sel: creatures(), n: Value::Const(4) }),
            effect: Effect::Move {
                what: Selector::RandomOf(Box::new(creatures())),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..legend(
            "Viconia, Drow Apostate",
            cost(&[generic(2), crate::mana::b()]),
            vec![CreatureType::Elf, CreatureType::Cleric],
            2,
            3,
        )
    }
}

/// Acquisition Octopus — {2}{U} 2/2 Equipment Octopus artifact creature; it
/// or the equipped creature dealing combat damage to a player draws a card.
/// Reconfigure {2}.
pub fn acquisition_octopus() -> CardDefinition {
    let draws = || TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: draw(1),
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Reconfigure(cost(&[generic(2)]))],
        triggered_abilities: vec![draws()],
        equipped_bonus: Some(EquipBonus { triggered_abilities: vec![draws()], ..Default::default() }),
        ..creature("Acquisition Octopus", cost(&[generic(2), u()]), vec![CreatureType::Octopus], 2, 2)
    }
}

/// Elfhame Druid — {1}{G} 0/2 Elf Druid. {T}: {G}. {T}: {G}{G}, spent only on
/// kicked spells.
pub fn elfhame_druid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            crate::sets::tap_add(Color::Green),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Restricted(
                        Box::new(crate::effect::ManaPayload::Colors(vec![Color::Green, Color::Green])),
                        crate::mana::SpendRestriction::KickedSpellsOnly,
                    ),
                },
                ..Default::default()
            },
        ],
        ..creature("Elfhame Druid", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Druid], 0, 2)
    }
}
