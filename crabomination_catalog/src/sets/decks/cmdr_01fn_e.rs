//! Commander: partner-pair closers, batch two (`01FnAVVR`). Tests in
//! `tests/recent_b/cmdr_01fn_a.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{blocks, etb, mentor, mint_token, myriad, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost, host: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(host) },
        ..Default::default()
    }
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn roll(sides: u8, results: Vec<(u8, u8, Effect)>) -> Effect {
    Effect::RollDie {
        sides,
        count: Value::ONE,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results,
        on_doubles: None,
    }
}

fn at_your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

fn move_this(to: ZoneDest) -> Effect {
    Effect::Move { what: Selector::This, to }
}

/// Loathsome Troll — {3}{G}{G} 6/2 Troll. {3}{G} from the graveyard: roll a
/// d20 — 1-9 to the library top, 10-19 to hand, 20 onto the battlefield tapped.
pub fn loathsome_troll() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            from_graveyard: true,
            effect: roll(
                20,
                vec![
                    (1, 9, move_this(ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top })),
                    (10, 19, move_this(ZoneDest::Hand(PlayerRef::You))),
                    (20, u8::MAX, move_this(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true })),
                ],
            ),
            ..Default::default()
        }],
        ..creature("Loathsome Troll", cost(&[generic(3), g(), g()]), vec![CreatureType::Troll], 6, 2)
    }
}

/// Sea Gate Colossus — {7} 7/5 Golem Warrior artifact creature; {1} less per
/// creature in your party.
pub fn sea_gate_colossus() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each creature in your party.",
            effect: StaticEffect::SelfCostReducedByValue { amount: Value::PartyCount },
        }],
        ..creature("Sea Gate Colossus", cost(&[generic(7)]), vec![CreatureType::Golem, CreatureType::Warrior], 7, 5)
    }
}

fn clown_robot() -> TokenDefinition {
    TokenDefinition {
        name: "Clown Robot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Clown, CreatureType::Robot], ..Default::default() },
        ..Default::default()
    }
}

/// Clowning Around — {1}{W} sorcery. Two 1/1 white Clown Robots, then a d6:
/// at or under your Robot count, a third.
pub fn clowning_around() -> CardDefinition {
    spell(
        "Clowning Around",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            mint_token(clown_robot(), 2),
            roll(
                6,
                vec![(
                    1,
                    u8::MAX,
                    Effect::If {
                        cond: Predicate::ValueAtMost(
                            Value::LastDieRoll,
                            Value::PermanentCountControlledByMatching(PlayerRef::You, R::HasCreatureType(CreatureType::Robot)),
                        ),
                        then: Box::new(mint_token(clown_robot(), 1)),
                        else_: Box::new(Effect::Noop),
                    },
                )],
            ),
        ]),
    )
}

/// Tabaxi Toucaneers — {4}{W} 3/2 flying, myriad Cat Ranger.
pub fn tabaxi_toucaneers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![myriad()],
        ..creature("Tabaxi Toucaneers", cost(&[generic(4), w()]), vec![CreatureType::Cat, CreatureType::Ranger], 3, 2)
    }
}

/// Auntie's Sentence — {1}{B} sorcery. Choose one: target opponent reveals
/// their hand and discards a nonland permanent card you choose; or target
/// creature gets -2/-2 until end of turn.
pub fn aunties_sentence() -> CardDefinition {
    spell(
        "Auntie's Sentence",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::DiscardChosen {
                from: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Nonland.and(R::PermanentCard),
            },
            pump(target_filtered(R::Creature), -2, -2),
        ]),
    )
}

/// Containment Protocol — {2}{U} Aura. ETB taps the enchanted creature; it
/// doesn't untap during its controller's untap step.
pub fn containment_protocol() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Tap { what: enchanted() })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: enchanted() },
        }],
        ..aura("Containment Protocol", cost(&[generic(2), u()]), R::Creature)
    }
}

/// So Tiny — {U} flash Aura. Enchanted creature gets -2/-0, or -6/-0 while
/// its controller has seven or more cards in their graveyard.
pub fn so_tiny() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets -2/-0.",
                effect: StaticEffect::PumpPT { applies_to: enchanted(), power: -2, toughness: 0 },
            },
            StaticAbility {
                description: "It gets -6/-0 instead as long as its controller has seven or more cards in their graveyard.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::ValueAtLeast(
                        Value::GraveyardSizeOf(PlayerRef::ControllerOf(Box::new(enchanted()))),
                        Value::Const(7),
                    ),
                    inner: Box::new(StaticEffect::PumpPT { applies_to: enchanted(), power: -4, toughness: 0 }),
                },
            },
        ],
        ..aura("So Tiny", cost(&[u()]), R::Creature)
    }
}

/// Venser's Diffusion — {2}{U} instant. Return target nonland permanent to
/// its owner's hand (the "or suspended card" target is not offered).
pub fn vensers_diffusion() -> CardDefinition {
    spell(
        "Venser's Diffusion",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Move {
            what: target_filtered(R::Permanent.and(R::Nonland)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
    )
}

/// Volcanic Awakening — {4}{R}{R} sorcery, storm. Destroy target land.
pub fn volcanic_awakening() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..spell(
            "Volcanic Awakening",
            cost(&[generic(4), r(), r()]),
            CardType::Sorcery,
            Effect::Destroy { what: target_filtered(R::Land) },
        )
    }
}

/// Biting Rain — {2}{B}{B} sorcery, madness {2}{B}. All creatures get -2/-2.
pub fn biting_rain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[generic(2), b()]))],
        ..spell(
            "Biting Rain",
            cost(&[generic(2), b(), b()]),
            CardType::Sorcery,
            pump(Selector::EachPermanent(R::Creature), -2, -2),
        )
    }
}

/// False Peace — {W} sorcery. Target player skips their next combat phase.
pub fn false_peace() -> CardDefinition {
    CardDefinition {
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: target_filtered(R::Player), amount: Value::ZERO },
            Effect::SkipNextCombatPhase { who: PlayerRef::Target(0) },
        ]),
        ..spell("False Peace", cost(&[w()]), CardType::Sorcery, Effect::Noop)
    }
}

/// Instruments of War — {4} flash artifact. Name a creature type as it
/// enters; your creatures of that type get +1/+1.
pub fn instruments_of_war() -> CardDefinition {
    CardDefinition {
        name: "Instruments of War",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +1/+1.",
            effect: StaticEffect::AnthemForChosenType {
                all_players: false,
                power: 1,
                toughness: 1,
                exclude_source: false,
                opponents: false,
                per_counter: None,
            },
        }],
        ..Default::default()
    }
}

/// Sunscour — {5}{W}{W} sorcery. Destroy all creatures; you may exile two
/// white cards from your hand rather than pay its mana cost.
pub fn sunscour() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            exile_filter: Some(R::HasColor(Color::White)),
            exile_filter_extra: 1,
            ..Default::default()
        }),
        ..spell("Sunscour", cost(&[generic(5), w(), w()]), CardType::Sorcery, Effect::Destroy {
            what: Selector::EachPermanent(R::Creature),
        })
    }
}

/// Elvish Farmer — {1}{G} 0/2 Elf. Spore counters each upkeep; three make a
/// Saproling; sacrificing a Saproling gains 2 life.
pub fn elvish_farmer() -> CardDefinition {
    let saproling = TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![at_your_upkeep(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Spore,
            amount: Value::ONE,
        })],
        activated_abilities: vec![
            ActivatedAbility {
                remove_counter_cost: Some((CounterType::Spore, 3)),
                effect: mint_token(saproling, 1),
                ..Default::default()
            },
            ActivatedAbility {
                sac_other_filter: Some((R::HasCreatureType(CreatureType::Saproling), 1)),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..creature("Elvish Farmer", cost(&[generic(1), g()]), vec![CreatureType::Elf], 0, 2)
    }
}

/// Biting Tether — {4}{U} Aura. You control enchanted creature; it takes a
/// -1/-1 counter at the beginning of your upkeep.
pub fn biting_tether() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::GainControlWhileSourceRemains { what: enchanted() }),
            at_your_upkeep(Effect::AddCounter { what: enchanted(), kind: CounterType::MinusOneMinusOne, amount: Value::ONE }),
        ],
        ..aura("Biting Tether", cost(&[generic(4), u()]), R::Creature)
    }
}

/// Wojek Bodyguard — {2}{R} 3/3 Human Soldier, mentor; can't attack or block
/// alone.
pub fn wojek_bodyguard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackOrBlockAlone],
        triggered_abilities: vec![mentor()],
        ..creature("Wojek Bodyguard", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 3)
    }
}

/// Vinebred Brawler — {2}{G} 4/2 Elf Berserker; must be blocked if able.
/// Attacking, another target Elf you control gets +2/+1.
pub fn vinebred_brawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustBeBlocked],
        triggered_abilities: vec![on_attack(pump(
            target_filtered(R::HasCreatureType(CreatureType::Elf).and(R::ControlledByYou).and(R::OtherThanSource)),
            2,
            1,
        ))],
        ..creature("Vinebred Brawler", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Berserker], 4, 2)
    }
}

/// Lys Alana Bowmaster — {2}{G} 2/2 reach Elf Archer. Casting an Elf spell,
/// it may deal 2 damage to target creature with flying.
pub fn lys_alana_bowmaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasCreatureType(CreatureType::Elf))),
            effect: Effect::MayDo {
                description: "Deal 2 damage to target creature with flying?".into(),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                    amount: Value::Const(2),
                }),
            },
        }],
        ..creature("Lys Alana Bowmaster", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Archer], 2, 2)
    }
}

/// Medicine Runner — {1}{G/W} 2/1 Elf Cleric. ETB: you may remove a counter
/// from target permanent.
pub fn medicine_runner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Remove a counter from target permanent?".into(),
            body: Box::new(Effect::RemoveAnyCounter { what: target_filtered(R::Permanent) }),
        })],
        ..creature(
            "Medicine Runner",
            ManaCost::new(vec![generic(1), crate::mana::hybrid(Color::Green, Color::White)]),
            vec![CreatureType::Elf, CreatureType::Cleric],
            2,
            1,
        )
    }
}

/// Clockwork Hydra — {5} 0/0 artifact Hydra entering with four +1/+1
/// counters. Attacking or blocking, it removes one to deal 1 damage to any
/// target; {T}: a +1/+1 counter on it.
pub fn clockwork_hydra() -> CardDefinition {
    let spit = || Effect::If {
        cond: Predicate::ValueAtLeast(Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne }, Value::ONE),
        then: Box::new(Effect::Seq(vec![
            Effect::RemoveCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::DealDamage { to: target_filtered(R::Any), amount: Value::ONE },
        ])),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        triggered_abilities: vec![on_attack(spit()), blocks(spit())],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Clockwork Hydra", cost(&[generic(5)]), vec![CreatureType::Hydra], 0, 0)
    }
}

/// Guardians of Koilos — {5} 4/4 Construct artifact creature. ETB: you may
/// return another target historic permanent you control to its owner's hand.
pub fn guardians_of_koilos() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return another historic permanent you control to hand?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(
                    R::Permanent
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource)
                        .and(R::historic()),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..creature("Guardians of Koilos", cost(&[generic(5)]), vec![CreatureType::Construct], 4, 4)
    }
}
