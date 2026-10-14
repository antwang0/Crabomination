//! Commander: one-card gaps of the 2026-10-14 ranks 1-5500 EDHREC rescan,
//! second file — each card is the last one a seated average deck was
//! missing. Tests in `tests/recent_b/cmdr_rescan_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility,
    PlaneswalkerSubtype, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{mint_token, on_attack, target_any, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use crabomination_base::tokens::food_token;
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

fn your_step(step: TurnStep) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl)
}

fn your_creatures() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou))
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        keywords,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn walker(name: &'static str, mana: ManaCost, subtype: PlaneswalkerSubtype, loyalty: u32, abilities: Vec<LoyaltyAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![subtype], ..Default::default() },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

fn loyalty(cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() }
}

/// Glorious Sunrise — {3}{G}{G} enchantment. At the beginning of combat on
/// your turn, choose one — team +1/+1 and trample; target land gains "{T}:
/// Add {G}{G}{G}" until end of turn; draw a card if you control a creature
/// with power 3 or greater; gain 3 life. (The Tarrasque.)
pub fn glorious_sunrise() -> CardDefinition {
    CardDefinition {
        name: "Glorious Sunrise",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::BeginCombat),
            effect: Effect::ChooseMode(vec![
                Effect::Seq(vec![
                    Effect::PumpPT { what: your_creatures(), power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: your_creatures(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                ]),
                Effect::GainActivatedAbility {
                    what: target_filtered(R::Land),
                    ability: Box::new(ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green; 3]) },
                        ..Default::default()
                    }),
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(3)),
                    )),
                    then: Box::new(draw(1)),
                    else_: Box::new(Effect::Noop),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ]),
        }],
        ..Default::default()
    }
}

/// Unforgiving One — {2}{B} 2/3 Spirit; menace. Whenever it attacks, return
/// target creature card with mana value X or less from your graveyard to the
/// battlefield, X the modified creatures you control. (Aron.)
pub fn unforgiving_one() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::Move {
            what: target_filtered(
                R::InYourGraveyard
                    .and(R::Creature)
                    .and(R::ManaValueAtMostYourCount(Box::new(R::Creature.and(R::IsModified)))),
            ),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        ..creature("Unforgiving One", cost(&[generic(2), b()]), vec![CreatureType::Spirit], 2, 3)
    }
}

/// Taeko, the Patient Avalanche — {3}{U} legendary 4/5 Turtle Ninja; enters
/// tapped. Whenever another creature you control leaves without dying, scry
/// 1 and a +1/+1 counter on Taeko. Whenever Taeko attacks, you may pay
/// {U/B}; when you do, target attacking creature can't be blocked this turn.
pub fn taeko_the_patient_avalanche() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Taeko enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureLeavesBattlefieldNotDying, EventScope::AnotherOfYours),
                effect: Effect::Seq(vec![
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                ]),
            },
            on_attack(Effect::MayPay {
                description: "Pay {U/B}?".into(),
                mana_cost: cost(&[hybrid(Color::Blue, Color::Black)]),
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::GrantKeyword {
                        what: target_filtered(R::Creature.and(R::IsAttacking)),
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    }),
                }),
                else_: None,
            }),
        ],
        ..legend("Taeko, the Patient Avalanche", cost(&[generic(3), u()]), vec![CreatureType::Turtle, CreatureType::Ninja], 4, 5)
    }
}

/// Shu Yun, the Silent Tempest — {2}{U} legendary 3/2 Human Monk; prowess.
/// Whenever you cast a noncreature spell, you may pay {R/W}{R/W}; if you do,
/// target creature gains double strike until end of turn.
pub fn shu_yun_the_silent_tempest() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_noncreature()),
            effect: Effect::MayPay {
                description: "Pay {R/W}{R/W}?".into(),
                mana_cost: cost(&[rw(), rw()]),
                body: Box::new(Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            },
        }],
        ..legend("Shu Yun, the Silent Tempest", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Monk], 3, 2)
    }
}

/// Meatsqueak, Hoard Lord — {2}{G} legendary 2/3 Squirrel. Whenever another
/// creature dies, create a Food (once each turn). For every seven Foods you
/// control, Squirrels you control get +3/+3.
pub fn meatsqueak_hoard_lord() -> CardDefinition {
    let foods = Value::count(Selector::EachPermanent(
        R::HasArtifactSubtype(ArtifactSubtype::Food).and(R::ControlledByYou),
    ));
    let n = Value::Times(Box::new(Value::DivDown(Box::new(foods), 7)), Box::new(Value::Const(3)));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf)))
                .once_per_turn(),
            effect: mint_token(food_token(), 1),
        }],
        static_abilities: vec![StaticAbility {
            description: "For every seven Foods you control, Squirrels you control get +3/+3.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Squirrel)).and(R::ControlledByYou),
                ),
                power: n.clone(),
                toughness: n,
            },
        }],
        ..legend("Meatsqueak, Hoard Lord", cost(&[generic(2), g()]), vec![CreatureType::Squirrel], 2, 3)
    }
}

/// Asari Captain — {3}{R}{W} 4/3 Human Samurai; haste. Whenever a Samurai or
/// Warrior you control attacks alone, it gets +1/+0 for each Samurai or
/// Warrior you control. (Risona.)
pub fn asari_captain() -> CardDefinition {
    let sw = R::HasCreatureType(CreatureType::Samurai).or(R::HasCreatureType(CreatureType::Warrior));
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::AttackingAlone,
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: sw.clone() },
            ])),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::count(Selector::EachPermanent(sw.and(R::Creature).and(R::ControlledByYou))),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Asari Captain", cost(&[generic(3), r(), w()]), vec![CreatureType::Human, CreatureType::Samurai], 4, 3)
    }
}

/// Weatherlight Compleated — {2} legendary 5/5 Vehicle; flying. With four or
/// more phyresis counters it's a Phyrexian creature. Whenever a creature you
/// control dies, a phyresis counter on it; then draw a card if it has seven
/// or more, else scry 1.
pub fn weatherlight_compleated() -> CardDefinition {
    let phyresis = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Phyresis };
    CardDefinition {
        name: "Weatherlight Compleated",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "As long as it has four or more phyresis counters on it, it's a Phyrexian creature in addition to its other types.",
            effect: StaticEffect::SelfIsCreatureIf {
                condition: Predicate::ValueAtLeast(phyresis(), Value::Const(4)),
                creature_types: vec![CreatureType::Phyrexian],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Phyresis, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(phyresis(), Value::Const(7)),
                    then: Box::new(draw(1)),
                    else_: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::ONE }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Tokka & Rahzar, Terrible Twos — {B/R}{B/R} legendary 3/2; can't be
/// countered; menace. Whenever a player casts a spell with less mana spent
/// than its mana value, they take 3 damage.
pub fn tokka_rahzar_terrible_twos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered, Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::Not(Box::new(Predicate::ValueAtLeast(
                    Value::CastSpellManaSpent,
                    Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                ))),
                then: Box::new(Effect::DealDamage { to: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(3) }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..legend(
            "Tokka & Rahzar, Terrible Twos",
            cost(&[hybrid(Color::Black, Color::Red), hybrid(Color::Black, Color::Red)]),
            vec![CreatureType::Turtle, CreatureType::Wolf, CreatureType::Mutant],
            3,
            2,
        )
    }
}

/// Najal, the Storm Runner — {2}{U}{U}{R} legendary 5/4 Efreet Wizard. You may
/// cast sorceries as though they had flash. Whenever Najal attacks, you may
/// pay {2}; if you do, copy your next instant or sorcery this turn.
pub fn najal_the_storm_runner() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may cast sorcery spells as though they had flash.",
            effect: StaticEffect::ControllerSorceriesAsFlash,
        }],
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "Pay {2}?".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::CopyNextInstantSorceryThisTurn { count: Value::ONE }),
            else_: None,
        })],
        ..legend(
            "Najal, the Storm Runner",
            cost(&[generic(2), u(), u(), r()]),
            vec![CreatureType::Efreet, CreatureType::Wizard],
            5,
            4,
        )
    }
}

/// Vikya, Scorching Stalwart — {2}{W} legendary 2/4 Human Warrior; training.
/// {4}{R}, {Q}, discard a card: damage equal to its power to any target; if
/// excess damage was dealt to a creature this way, draw a card.
pub fn vikya_scorching_stalwart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::training()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r()]),
            untap_self_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::PowerOf(Box::new(Selector::This)) },
                Effect::If {
                    cond: Predicate::All(vec![
                        Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                        Predicate::ExcessDamageDealtThisResolution,
                    ]),
                    then: Box::new(draw(1)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend("Vikya, Scorching Stalwart", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 4)
    }
}

/// Elsewhere Flask — {2} artifact. Enters: draw a card. Sacrifice it: lands
/// you control become the basic land type of your choice until end of turn.
/// (Gerrard, Weatherlight Hero.)
pub fn elsewhere_flask() -> CardDefinition {
    CardDefinition {
        name: "Elsewhere Flask",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![crate::effect::shortcut::etb(draw(1))],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::LandsBecomeChosenBasicType {
                what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                duration: Duration::EndOfTurn,
                from_chosen_basic: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Moon Girl and Devil Dinosaur — {1}{G}{U} legendary 2/2. Your second draw
/// each turn makes them a 6/6 trampler until end of turn. Whenever an
/// artifact you control enters, draw a card (once each turn).
pub fn moon_girl_and_devil_dinosaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::SetBasePT { what: Selector::This, power: Value::Const(6), toughness: Value::Const(6), duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact })
                    .once_per_turn(),
                effect: draw(1),
            },
        ],
        ..legend(
            "Moon Girl and Devil Dinosaur",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Human, CreatureType::Dinosaur, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Chief Magistrate of Mercadia — {4}{R}{W} legendary 3/3 Human Noble. ETB:
/// you become the monarch. Upkeep: a 2/1 red haste Goblin; then if you're
/// the monarch, a copy of each creature token you control.
pub fn chief_magistrate_of_mercadia() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            TriggeredAbility {
                event: your_step(TurnStep::Upkeep),
                effect: Effect::Seq(vec![
                    mint_token(token("Goblin", 2, 1, vec![Color::Red], vec![CreatureType::Goblin], vec![Keyword::Haste]), 1),
                    Effect::If {
                        cond: Predicate::IsMonarch { who: PlayerRef::You },
                        then: Box::new(Effect::ForEach {
                            selector: Selector::EachPermanent(R::Creature.and(R::IsToken).and(R::ControlledByYou)),
                            body: Box::new(Effect::CreateTokenCopyOf {
                                who: PlayerRef::You,
                                count: Value::ONE,
                                source: Selector::TriggerSource,
                                extra_creature_types: vec![],
                                extra_card_types: vec![],
                                override_pt: None,
                                override_colors: None,
                                enters_tapped: false,
                                non_legendary: false,
                                legendary: false,
                                extra_keywords: vec![],
                                no_mana_cost: false,
                                enters_with_counters: None,
                                remove_keywords: vec![],
                            }),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..legend(
            "Chief Magistrate of Mercadia",
            cost(&[generic(4), r(), w()]),
            vec![CreatureType::Human, CreatureType::Noble],
            3,
            3,
        )
    }
}

/// Lightmine Field — {2}{W}{W} enchantment. Whenever one or more creatures
/// attack, it deals damage to each attacker equal to the number attacking.
/// (Basandra.)
pub fn lightmine_field() -> CardDefinition {
    let attackers = || Selector::EachPermanent(R::Creature.and(R::IsAttacking));
    CardDefinition {
        name: "Lightmine Field",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer),
            effect: Effect::DealDamage { to: attackers(), amount: Value::count(attackers()) },
        }],
        ..Default::default()
    }
}

/// Thawing Glaciers — land; enters tapped. {1}, {T}: search for a basic land,
/// put it onto the battlefield tapped, then shuffle; return this land to its
/// owner's hand at the beginning of the next cleanup step. (Taniwha.)
pub fn thawing_glaciers() -> CardDefinition {
    CardDefinition {
        name: "Thawing Glaciers",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextCleanupStep,
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Creeping Crystal Coating — {2}{G} Aura; flash. Enchanted creature gets
/// +0/+3 and has "Whenever this creature attacks, create a Food token."
/// (Bosco.)
pub fn creeping_crystal_coating() -> CardDefinition {
    CardDefinition {
        name: "Creeping Crystal Coating",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            toughness: 3,
            triggered_abilities: vec![on_attack(mint_token(food_token(), 1))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Thomil, the Destroyer — {3}{B}{B} legendary planeswalker, loyalty 4; can
/// be your commander. +2: a 2/2 black Zombie. 0: you may sacrifice a
/// creature; if you do, add {B}{B}{B}. −5: a Lord of the Pit token.
pub fn thomil_the_destroyer() -> CardDefinition {
    let pit = TokenDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::Upkeep),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                )),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
                else_: Box::new(Effect::DealDamage { to: Selector::You, amount: Value::Const(7) }),
            },
        }],
        ..token("Lord of the Pit", 7, 7, vec![Color::Black], vec![CreatureType::Demon], vec![Keyword::Flying, Keyword::Trample])
    };
    CardDefinition {
        can_be_commander: true,
        ..walker(
            "Thomil, the Destroyer",
            cost(&[generic(3), b(), b()]),
            PlaneswalkerSubtype::Thomil,
            4,
            vec![
                loyalty(2, mint_token(token("Zombie", 2, 2, vec![Color::Black], vec![CreatureType::Zombie], vec![]), 1)),
                loyalty(
                    0,
                    Effect::MaySacrifice {
                        description: "Sacrifice a creature for {B}{B}{B}?".into(),
                        filter: R::Creature,
                        count: Value::ONE,
                        then: Box::new(Effect::AddMana {
                            who: PlayerRef::You,
                            pool: ManaPayload::Colors(vec![Color::Black; 3]),
                        }),
                        else_: None,
                    },
                ),
                loyalty(-5, mint_token(pit, 1)),
            ],
        )
    }
}

/// Ajani, Outland Chaperone — {1}{W}{W} legendary planeswalker, loyalty 3.
/// +1: a 1/1 green and white Kithkin. −2: 4 damage to target tapped creature.
/// −8: look at the top X (your life), put any number of nonland permanent
/// cards with mana value 3 or less onto the battlefield, then shuffle.
/// (Wilfred Mott.)
pub fn ajani_outland_chaperone() -> CardDefinition {
    let life = Value::LifeOf(PlayerRef::You);
    walker(
        "Ajani, Outland Chaperone",
        cost(&[generic(1), w(), w()]),
        PlaneswalkerSubtype::Ajani,
        3,
        vec![
            loyalty(1, mint_token(token("Kithkin", 1, 1, vec![Color::Green, Color::White], vec![CreatureType::Kithkin], vec![]), 1)),
            loyalty(-2, Effect::DealDamage { to: target_filtered(R::Creature.and(R::Tapped)), amount: Value::Const(4) }),
            loyalty(
                -8,
                Effect::Seq(vec![
                    Effect::LookPickToHand(Box::new(LookPick {
                        who: PlayerRef::You,
                        count: life.clone(),
                        take: Some(life),
                        pick_filter: Some(R::PermanentCard.and(R::Nonland).and(R::ManaValueAtMost(3))),
                        to_battlefield: true,
                        optional: true,
                        ..Default::default()
                    })),
                    Effect::ShuffleLibrary { who: PlayerRef::You },
                ]),
            ),
        ],
    )
}

/// See the Unwritten — {4}{G}{G} sorcery. Reveal the top eight; you may put
/// a creature card from among them onto the battlefield (two with ferocious);
/// the rest go to your graveyard. (Hua Tuo.)
pub fn see_the_unwritten() -> CardDefinition {
    CardDefinition {
        name: "See the Unwritten",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(8),
            pick_filter: Some(R::Creature),
            take: Some(Value::IfPred {
                pred: Box::new(Predicate::FerociousActive { who: PlayerRef::You }),
                then: Box::new(Value::Const(2)),
                else_: Box::new(Value::ONE),
            }),
            to_battlefield: true,
            rest_to_graveyard: true,
            optional: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Balefang the Unslayable — {3}{R}{R} legendary 5/5 Demon Dragon; flying,
/// trample, protection from Angels and from Archons. Players can't gain
/// life. ETB: target opponent creates a tapped Baneslayer Angel token,
/// goaded for the rest of the game. (The token's mana value reads 0.)
pub fn balefang_the_unslayable() -> CardDefinition {
    let angel = TokenDefinition {
        tapped: true,
        ..token(
            "Baneslayer Angel",
            5,
            5,
            vec![Color::White],
            vec![CreatureType::Angel],
            vec![
                Keyword::Flying,
                Keyword::FirstStrike,
                Keyword::Lifelink,
                Keyword::ProtectionFromCreatureType(CreatureType::Demon),
                Keyword::ProtectionFromCreatureType(CreatureType::Dragon),
            ],
        )
    };
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Trample,
            Keyword::ProtectionFromCreatureType(CreatureType::Angel),
            Keyword::ProtectionFromCreatureType(CreatureType::Archon),
        ],
        static_abilities: vec![StaticAbility {
            description: "Players can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife { target: crate::effect::PlayerStaticTarget::EachPlayer },
        }],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 1,
            filter: R::OpponentPlayer,
            effect: Box::new(Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::Target(0), count: Value::ONE, definition: Arc::new(angel) },
                Effect::GoadForTheGame { what: Selector::LastCreatedToken },
            ])),
        })],
        ..legend("Balefang the Unslayable", cost(&[generic(3), r(), r()]), vec![CreatureType::Demon, CreatureType::Dragon], 5, 5)
    }
}

/// Kitsune, Dragon's Daughter — {4}{U}{U} legendary 6/6 Fox Warlock Avatar;
/// vigilance. Whenever she enters or deals combat damage to a player, you may
/// exchange control of two other target creatures controlled by different
/// players.
pub fn kitsune_dragons_daughter() -> CardDefinition {
    let swap = || Effect::MayDo {
        description: "Exchange control of the two creatures?".into(),
        body: Box::new(Effect::ExchangeControl {
            a: target_filtered(R::Creature.and(R::OtherThanSource)),
            b: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature.and(R::OtherThanSource).and(R::DifferentControllerFromTargetSlot(0)),
            },
        }),
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(swap()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: swap(),
            },
        ],
        ..legend(
            "Kitsune, Dragon's Daughter",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Fox, CreatureType::Warlock, CreatureType::Avatar],
            6,
            6,
        )
    }
}

/// Captain Marvel, Earth's Protector — {3}{W}{W} legendary 5/4 Human Kree
/// Hero; flash, flying, lifelink. Power-up {5}{W}{W}: a +1/+1 and an
/// indestructible counter on her.
pub fn captain_marvel_earths_protector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w(), w()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Indestructible, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Captain Marvel, Earth's Protector",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Kree, CreatureType::Hero],
            5,
            4,
        )
    }
}

/// Flying Drone — {1}{U} 1/2 artifact Robot Scout; flying, vigilance. {1}{U},
/// {T}: draw, then discard; it costs {1}{U} less if another creature with
/// flying entered under your control this turn — written as the full-price
/// loot gated off that and a free one gated on it. (Vulture.)
pub fn flying_drone() -> CardDefinition {
    let flyer_entered = Predicate::CreatureEnteredThisTurnMatching {
        who: PlayerRef::You,
        filter: R::HasKeyword(Keyword::Flying).and(R::OtherThanSource),
    };
    let loot = || {
        Effect::Seq(vec![
            draw(1),
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ])
    };
    CardDefinition {
        name: "Flying Drone",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot, CreatureType::Scout], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1), u()]),
                effect: loot(),
                condition: Some(Predicate::Not(Box::new(flyer_entered.clone()))),
                ..Default::default()
            },
            ActivatedAbility { tap_cost: true, effect: loot(), condition: Some(flyer_entered), ..Default::default() },
        ],
        ..Default::default()
    }
}

/// You Cannot Pass! — {W} instant. Destroy target creature that blocked or
/// was blocked by a legendary creature this turn. (Senu.)
pub fn you_cannot_pass() -> CardDefinition {
    CardDefinition {
        name: "You Cannot Pass!",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.and(R::BlockedOrWasBlockedByThisTurn(Box::new(
                R::Creature.and(R::HasSupertype(Supertype::Legendary)),
            )))),
        },
        ..Default::default()
    }
}

/// Selenia, the Cursed Heart — {4}{W}{B} legendary 2/5 Angel Warrior; flying,
/// lifelink. If you would gain life, you gain twice that much. When she dies,
/// create Selenia's Curse — a legendary black Aura Curse token attached to
/// target opponent: "If enchanted player would lose life, they lose twice
/// that much life instead."
pub fn selenia_the_cursed_heart() -> CardDefinition {
    use crate::effect::PlayerStaticTarget;
    let curse = TokenDefinition {
        name: "Selenia's Curse".into(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "If enchanted player would lose life, they lose twice that much life instead.",
            effect: StaticEffect::LifeLossMultiplier { target: PlayerStaticTarget::EnchantedPlayer, factor: 2 },
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "If you would gain life, you gain twice that much life instead.",
            effect: StaticEffect::LifeGainMultiplier { target: PlayerStaticTarget::Controller, factor: 2 },
        }],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(curse) },
            Effect::Attach { what: Selector::LastCreatedToken, to: target_filtered(R::OpponentPlayer) },
        ]))],
        ..legend(
            "Selenia, the Cursed Heart",
            cost(&[generic(4), w(), b()]),
            vec![CreatureType::Angel, CreatureType::Warrior],
            2,
            5,
        )
    }
}

/// Trove of Temptation — {3}{R} enchantment. Each opponent must attack you or
/// a planeswalker you control with at least one creature each combat if able.
/// At the beginning of your end step, create a Treasure token. (Kazuul.)
pub fn trove_of_temptation() -> CardDefinition {
    CardDefinition {
        name: "Trove of Temptation",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Each opponent must attack you or a planeswalker you control with at least one creature each combat if able.",
            effect: StaticEffect::OpponentsMustAttackYouWithAtLeastOne,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End),
            effect: mint_token(crabomination_base::tokens::treasure_token(), 1),
        }],
        ..Default::default()
    }
}

/// Wardscale Dragon — {4}{W}{W} 4/4 Dragon; flying. As long as it is
/// attacking, defending player can't cast spells. (Dromoka, the Eternal.)
pub fn wardscale_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is attacking, defending player can't cast spells.",
            effect: StaticEffect::DefendingPlayerCantCastWhileAttacking,
        }],
        ..creature("Wardscale Dragon", cost(&[generic(4), w(), w()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Ashes of the Abhorrent — {1}{W} enchantment. Players can't cast spells
/// from graveyards or activate abilities of cards in graveyards. Whenever a
/// creature dies, you gain 1 life. (Baird.)
pub fn ashes_of_the_abhorrent() -> CardDefinition {
    CardDefinition {
        name: "Ashes of the Abhorrent",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Players can't cast spells from graveyards or activate abilities of cards in graveyards.",
            effect: StaticEffect::GraveyardCastAndActivationLock,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Ormos, Archive Keeper — {4}{U}{U} legendary 5/5 Sphinx; flying. If you
/// would draw from an empty library, instead put five +1/+1 counters on
/// Ormos. {1}{U}{U}, discard three cards with different names: draw five.
pub fn ormos_archive_keeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card while your library has no cards in it, instead put five +1/+1 counters on Ormos.",
            effect: StaticEffect::CountersInsteadOfDrawFromEmpty { kind: CounterType::PlusOnePlusOne, amount: 5 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u(), u()]),
            discard_cost: Some((R::Any, 3)),
            discard_cost_distinct_names: true,
            effect: draw(5),
            ..Default::default()
        }],
        ..legend("Ormos, Archive Keeper", cost(&[generic(4), u(), u()]), vec![CreatureType::Sphinx], 5, 5)
    }
}
