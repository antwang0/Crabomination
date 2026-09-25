//! Commander: the cards the **Elven Empire** precon (KHC, Lathril, Blade of
//! the Elves) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_lathril.rs`.
//!
//! Residuals (each also on its card):
//! - **Serpent's Soul-Jar** — every creature card it exiled is castable
//!   until end of turn, not one of them.
//! - **Roots of Wisdom** — the returned card comes from among the three
//!   milled, not from anywhere in the graveyard.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, fabricate, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, ty: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![ty], effect, ..Default::default() }
}

fn elf() -> R {
    R::HasCreatureType(CreatureType::Elf)
}

/// The number of Elves you control.
fn elves() -> Value {
    Value::CountOf(Box::new(Selector::EachPermanent(elf().and(R::Creature).and(R::ControlledByYou))))
}

fn elf_warrior() -> TokenDefinition {
    TokenDefinition {
        name: "Elf Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    }
}

fn elf_warriors(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(elf_warrior()) }
}

/// "Whenever an Elf you control dies" (`include_self` counts the source).
fn on_elf_dies(include_self: bool, effect: Effect) -> TriggeredAbility {
    let filter = if include_self { elf() } else { elf().and(R::OtherThanSource) };
    TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

/// Abomination of Llanowar — vigilance, menace; `*/*` = Elves you control
/// plus Elf cards in your graveyard.
pub fn abomination_of_llanowar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        dynamic_pt: Some(DynamicPt::CreaturesOfTypeControlledAndInGraveyard { creature_type: CreatureType::Elf }),
        ..legendary(creature(
            "Abomination of Llanowar",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Elf, CreatureType::Horror],
            0,
            0,
        ))
    }
}

/// Bounty of Skemfar — from the top six, up to one land onto the battlefield
/// tapped and up to one Elf card to hand; the rest on the bottom at random.
pub fn bounty_of_skemfar() -> CardDefinition {
    let elf_pick = |count: i32| {
        Box::new(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(count),
            pick_filter: Some(elf()),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })))
    };
    spell(
        "Bounty of Skemfar",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        // The land pick leaves the rest on top in order; the Elf pick then
        // looks at what is left of the six.
        Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(6),
            pick_filter: Some(R::Land),
            optional: true,
            picked_lands_to_battlefield: true,
            rest_on_top: true,
            then_if_picked: Some(elf_pick(5)),
            then_if_not_picked: Some(elf_pick(6)),
            ..Default::default()
        })),
    )
}

/// Crown of Skemfar — enchanted creature gets +1/+1 per Elf you control and
/// has reach; {2}{G}: return it from your graveyard to your hand.
pub fn crown_of_skemfar() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Crown of Skemfar",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +1/+1 for each Elf you control.",
                effect: StaticEffect::PumpPTByValue { applies_to: host(), power: elves(), toughness: elves() },
            },
            StaticAbility {
                description: "Enchanted creature has reach.",
                effect: StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::Reach },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cultivator of Blades — fabricate 2; attacking, it may pump the other
/// attackers by its power.
pub fn cultivator_of_blades() -> CardDefinition {
    let power = || Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![
            fabricate(2),
            on_attack(Effect::MayDo {
                description: "Other attacking creatures get +X/+X?".into(),
                body: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
                    power: power(),
                    toughness: power(),
                    duration: Duration::EndOfTurn,
                }),
            }),
        ],
        ..creature(
            "Cultivator of Blades",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Artificer],
            1,
            1,
        )
    }
}

/// Elderfang Ritualist — dying, it returns another Elf card from your
/// graveyard to your hand.
pub fn elderfang_ritualist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Move {
            what: target_filtered(elf().from_your_graveyard().and(R::OtherThanSource)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Elderfang Ritualist",
            cost(&[generic(2), b()]),
            vec![CreatureType::Elf, CreatureType::Cleric],
            3,
            1,
        )
    }
}

/// Elderfang Venom — attacking Elves you control have deathtouch; each Elf of
/// yours dying drains each opponent for 1.
pub fn elderfang_venom() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Attacking Elves you control have deathtouch.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(elf().and(R::ControlledByYou).and(R::IsAttacking)),
                keyword: Keyword::Deathtouch,
            },
        }],
        triggered_abilities: vec![on_elf_dies(
            true,
            Effect::Drain { from: Selector::Player(PlayerRef::EachOpponent), to: Selector::You, amount: Value::ONE },
        )],
        ..spell("Elderfang Venom", cost(&[generic(2), b(), g()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Elven Ambush — a 1/1 Elf Warrior per Elf you control.
pub fn elven_ambush() -> CardDefinition {
    spell("Elven Ambush", cost(&[generic(3), g()]), CardType::Instant, elf_warriors(elves()))
}

/// Eyeblight Cullers — dying, three 1/1 Elf Warriors, then mill three.
pub fn eyeblight_cullers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            elf_warriors(Value::Const(3)),
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
        ]))],
        ..creature(
            "Eyeblight Cullers",
            cost(&[generic(4), b()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Eyeblight Massacre — non-Elf creatures get -2/-2 until end of turn.
pub fn eyeblight_massacre() -> CardDefinition {
    spell(
        "Eyeblight Massacre",
        cost(&[generic(2), b(), b()]),
        CardType::Sorcery,
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(elf().negate())),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Golgari Findbroker — entering, return a permanent card from your graveyard
/// to your hand.
pub fn golgari_findbroker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::PermanentCard.from_your_graveyard()),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Golgari Findbroker",
            cost(&[b(), b(), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Shaman],
            3,
            4,
        )
    }
}

/// Jagged-Scar Archers — `*/*` = Elves you control; {T}: damage equal to its
/// power to a creature with flying.
pub fn jagged_scar_archers() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::CreaturesOfTypeControlled { creature_type: CreatureType::Elf }),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamageEqualToPower {
                source: Selector::This,
                target: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            },
            ..Default::default()
        }],
        ..creature(
            "Jagged-Scar Archers",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Archer],
            0,
            0,
        )
    }
}

/// Lys Alana Scarblade — {T}, discard an Elf card: a creature gets -X/-X,
/// X = Elves you control.
pub fn lys_alana_scarblade() -> CardDefinition {
    let minus = || Value::Negate(Box::new(elves()));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((elf(), 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: minus(),
                toughness: minus(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Lys Alana Scarblade",
            cost(&[generic(2), b()]),
            vec![CreatureType::Elf, CreatureType::Assassin],
            1,
            1,
        )
    }
}

/// Miara, Thorn of the Glade — each Elf of yours dying (Miara too) may pay
/// {1} and 1 life to draw. Partner.
pub fn miara_thorn_of_the_glade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![on_elf_dies(
            true,
            // CR 119.4 — paying life is losing it; at 0 life the player is
            // already gone, so the "can pay" check is the mana alone.
            Effect::MayPay {
                description: "Pay {1} and 1 life to draw a card?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ])),
                else_: None,
            },
        )],
        ..legendary(creature(
            "Miara, Thorn of the Glade",
            cost(&[generic(1), b()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            1,
            2,
        ))
    }
}

/// Numa, Joraga Chieftain — at the beginning of combat on your turn, pay
/// {X}{X} to distribute X +1/+1 counters among Elves. Partner.
pub fn numa_joraga_chieftain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::MayPayXTimes {
                times: 2,
                description: "Pay {X}{X} to distribute X +1/+1 counters among Elves?".into(),
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::DistributeCounters {
                        total: Value::XFromCost,
                        counter: CounterType::PlusOnePlusOne,
                        filter: elf().and(R::Creature),
                        max_targets: 5,
                    }),
                }),
            },
        }],
        ..legendary(creature(
            "Numa, Joraga Chieftain",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            2,
            2,
        ))
    }
}

/// Poison the Cup — destroy target creature; foretold, scry 2 too.
pub fn poison_the_cup() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[generic(1), b()])),
        ..spell(
            "Poison the Cup",
            cost(&[generic(1), b(), b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature) },
                Effect::If {
                    cond: Predicate::CastForetold,
                    then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Pride of the Perfect — Elves you control get +2/+0.
pub fn pride_of_the_perfect() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Elves you control get +2/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(elf().and(R::Creature).and(R::ControlledByYou)),
                power: 2,
                toughness: 0,
            },
        }],
        ..spell("Pride of the Perfect", cost(&[generic(3), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Prowess of the Fair — Kindred Enchantment — Elf: another nontoken Elf of
/// yours dying may make a 1/1 Elf Warrior.
pub fn prowess_of_the_fair() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: elf().and(R::NotToken) },
            ),
            effect: Effect::MayDo {
                description: "Create a 1/1 Elf Warrior?".into(),
                body: Box::new(elf_warriors(Value::ONE)),
            },
        }],
        ..spell("Prowess of the Fair", cost(&[generic(1), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Return Upon the Tide — reanimate a creature card of yours; an Elf brings
/// two 1/1 Elf Warriors. Foretell {3}{B}.
pub fn return_upon_the_tide() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[generic(3), b()])),
        ..spell(
            "Return Upon the Tide",
            cost(&[generic(4), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.from_your_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: elf() },
                    then: Box::new(elf_warriors(Value::Const(2))),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Rhys the Exiled — attacking gains a life per Elf you control; {B},
/// sacrifice an Elf: regenerate it.
pub fn rhys_the_exiled() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::GainLife { who: Selector::You, amount: elves() })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((elf(), 1)),
            sac_other_may_be_source: true,
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..legendary(creature(
            "Rhys the Exiled",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            3,
            2,
        ))
    }
}

/// Roots of Wisdom — mill three, then a land or Elf card to hand; if you
/// can't, draw a card.
///
/// ⚠ Residual: the card comes from among the three milled.
pub fn roots_of_wisdom() -> CardDefinition {
    spell(
        "Roots of Wisdom",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::MillThenToHand {
            amount: Value::Const(3),
            filter: R::Land.or(elf()),
            otherwise: Some(Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE })),
        },
    )
}

/// Ruthless Winnower — each player's upkeep, that player sacrifices a non-Elf
/// creature.
pub fn ruthless_winnower() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ActivePlayer),
                count: Value::ONE,
                filter: R::Creature.and(elf().negate()),
            },
        }],
        ..creature(
            "Ruthless Winnower",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Elf, CreatureType::Rogue],
            4,
            4,
        )
    }
}

/// Serpent's Soul-Jar — your dying Elves are exiled with it; {T}, pay 2 life:
/// cast creature spells from among them this turn.
///
/// ⚠ Residual: every exiled creature card becomes castable, not one.
pub fn serpents_soul_jar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_elf_dies(true, Effect::ExileWithSource { what: Selector::TriggerSource })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 2,
            effect: Effect::GrantMayPlay {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Exile,
                    filter: R::ExiledWithSource.and(R::Creature),
                },
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
            ..Default::default()
        }],
        ..spell("Serpent's Soul-Jar", cost(&[generic(2), b()]), CardType::Artifact, Effect::Noop)
    }
}

/// Skemfar Elderhall — enters tapped, taps for {G}; {2}{B}{B}{G}, {T},
/// sacrifice: up to one creature you don't control gets -2/-2, and two 1/1
/// Elf Warriors. Sorcery speed.
pub fn skemfar_elderhall() -> CardDefinition {
    CardDefinition {
        name: "Skemfar Elderhall",
        card_types: vec![CardType::Land],
        static_abilities: vec![super::super::enters_tapped()],
        activated_abilities: vec![
            super::super::tap_add(Color::Green),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b(), b(), g()]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::Creature.and(R::ControlledByYou.negate()),
                        effect: Box::new(Effect::PumpPT {
                            what: Selector::Target(0),
                            power: Value::Const(-2),
                            toughness: Value::Const(-2),
                            duration: Duration::EndOfTurn,
                        }),
                    },
                    elf_warriors(Value::Const(2)),
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Tergrid's Shadow — each player sacrifices two creatures. Foretell {2}{B}{B}.
pub fn tergrids_shadow() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[generic(2), b(), b()])),
        ..spell(
            "Tergrid's Shadow",
            cost(&[generic(3), b(), b()]),
            CardType::Instant,
            Effect::Sacrifice { who: Selector::Player(PlayerRef::EachPlayer), count: Value::Const(2), filter: R::Creature },
        )
    }
}

/// Twinblade Assassins — your end step draws a card if a creature died this
/// turn.
pub fn twinblade_assassins() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Twinblade Assassins",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Elf, CreatureType::Assassin],
            5,
            4,
        )
    }
}

/// Wolverine Riders — a 1/1 Elf Warrior each upkeep; another Elf of yours
/// entering gains you life equal to its toughness.
pub fn wolverine_riders() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
                effect: elf_warriors(Value::ONE),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: elf() }),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                },
            },
        ],
        ..creature(
            "Wolverine Riders",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            4,
            4,
        )
    }
}
