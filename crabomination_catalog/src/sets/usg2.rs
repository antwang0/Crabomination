//! Urza's Saga (USG) gap closure, second wave. Tests in `classic_sets/usg2`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword,
    LandType, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::card::{Predicate, WardCost};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{etb, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

use super::{tap_add, tap_add_colorless};

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

fn artifact(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn instant(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn cycling_two() -> Keyword {
    Keyword::Cycling(cost(&[generic(2)]))
}

fn enters_tapped() -> StaticAbility {
    StaticAbility {
        description: "This land enters tapped.",
        effect: StaticEffect::EntersTapped { applies_to: Selector::This },
    }
}

fn regenerate_for(c: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::Regenerate { what: Selector::This },
        ..Default::default()
    }
}

/// "At the beginning of your upkeep, sacrifice this unless you pay `c`."
fn upkeep_sacrifice_unless_pay(c: crate::mana::ManaCost) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::Upkeep),
            EventScope::YourControl,
        ),
        effect: Effect::UnlessPlayerPays {
            who: PlayerRef::You,
            cost: WardCost::Mana(c),
            then: Box::new(Effect::SacrificeSource),
        },
    }
}

/// "At the beginning of your upkeep, you may put a verse counter on this."
fn verse_upkeep() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::Upkeep),
            EventScope::YourControl,
        ),
        effect: Effect::MayDo {
            description: "Put a verse counter on this".into(),
            body: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Verse,
                amount: Value::ONE,
            }),
        },
    }
}

fn verses() -> Value {
    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Verse }
}

/// An Aura that enchants a creature for `bonus`.
fn creature_aura(name: &'static str, c: crate::mana::ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// "When this Aura is put into a graveyard from the battlefield, return it to
/// its owner's hand" — the USG Halo/Embrace recursion cycle.
fn returns_to_hand() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
        effect: Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
        },
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

fn cycling_tapland(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        keywords: vec![cycling_two()],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(color)],
        ..Default::default()
    }
}

/// Blasted Landscape — the colorless cycler; it enters untapped.
pub fn blasted_landscape() -> CardDefinition {
    CardDefinition {
        name: "Blasted Landscape",
        card_types: vec![CardType::Land],
        keywords: vec![cycling_two()],
        activated_abilities: vec![tap_add_colorless()],
        ..Default::default()
    }
}

pub fn drifting_meadow() -> CardDefinition {
    cycling_tapland("Drifting Meadow", Color::White)
}
pub fn polluted_mire() -> CardDefinition {
    cycling_tapland("Polluted Mire", Color::Black)
}
pub fn remote_isle() -> CardDefinition {
    cycling_tapland("Remote Isle", Color::Blue)
}
pub fn slippery_karst() -> CardDefinition {
    cycling_tapland("Slippery Karst", Color::Green)
}
pub fn smoldering_crater() -> CardDefinition {
    cycling_tapland("Smoldering Crater", Color::Red)
}

/// Shivan Gorge — a legendary land that pings the table.
pub fn shivan_gorge() -> CardDefinition {
    CardDefinition {
        name: "Shivan Gorge",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Thran Quarry — any colour, but it needs a creature to stand on.
pub fn thran_quarry() -> CardDefinition {
    CardDefinition {
        name: "Thran Quarry",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(
                Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
            )))),
            effect: Effect::SacrificeSource,
        }],
        activated_abilities: vec![super::tap_add_any_color()],
        ..Default::default()
    }
}

// ── Echo bodies ─────────────────────────────────────────────────────────────

/// Citanul Centaurs — {3}{G} 6/3 with shroud and echo.
pub fn citanul_centaurs() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud, Keyword::Echo(cost(&[generic(3), g()]))],
        ..creature("Citanul Centaurs", cost(&[generic(3), g()]), vec![CreatureType::Centaur], 6, 3)
    }
}

/// Herald of Serra — {2}{W}{W} 3/4 flier with vigilance and echo.
pub fn herald_of_serra() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Vigilance,
            Keyword::Echo(cost(&[generic(2), w(), w()])),
        ],
        ..creature("Herald of Serra", cost(&[generic(2), w(), w()]), vec![CreatureType::Angel], 3, 4)
    }
}

/// Lightning Dragon — {2}{R}{R} 4/4 flier with echo and a firebreathing line.
pub fn lightning_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Echo(cost(&[generic(2), r(), r()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Lightning Dragon", cost(&[generic(2), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Shivan Raptor — {2}{R} 3/1 with first strike, haste and echo.
pub fn shivan_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Haste,
            Keyword::Echo(cost(&[generic(2), r()])),
        ],
        ..creature("Shivan Raptor", cost(&[generic(2), r()]), vec![CreatureType::Dinosaur], 3, 1)
    }
}

/// Viashino Outrider — {2}{R} 4/3 with echo.
pub fn viashino_outrider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Echo(cost(&[generic(2), r()]))],
        ..creature("Viashino Outrider", cost(&[generic(2), r()]), vec![CreatureType::Lizard], 4, 3)
    }
}

/// Vug Lizard — {1}{R}{R} 3/4 mountainwalker with echo.
pub fn vug_lizard() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Landwalk(LandType::Mountain),
            Keyword::Echo(cost(&[generic(1), r(), r()])),
        ],
        ..creature("Vug Lizard", cost(&[generic(1), r(), r()]), vec![CreatureType::Lizard], 3, 4)
    }
}

/// Winding Wurm — {4}{G} 6/6 with echo.
pub fn winding_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Echo(cost(&[generic(4), g()]))],
        ..creature("Winding Wurm", cost(&[generic(4), g()]), vec![CreatureType::Wurm], 6, 6)
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Voice of Grace — {3}{W} 2/2 flier with protection from black.
pub fn voice_of_grace() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Black)],
        ..creature("Voice of Grace", cost(&[generic(3), w()]), vec![CreatureType::Angel], 2, 2)
    }
}

/// Voice of Law — {3}{W} 2/2 flier with protection from red.
pub fn voice_of_law() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Red)],
        ..creature("Voice of Law", cost(&[generic(3), w()]), vec![CreatureType::Angel], 2, 2)
    }
}

/// Zephid — {4}{U}{U} 3/4 flier with shroud.
pub fn zephid() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Shroud],
        ..creature("Zephid", cost(&[generic(4), u(), u()]), vec![CreatureType::Illusion], 3, 4)
    }
}

/// Viashino Runner — {3}{R} 3/2 with menace.
pub fn viashino_runner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        ..creature("Viashino Runner", cost(&[generic(3), r()]), vec![CreatureType::Lizard], 3, 2)
    }
}

/// Treetop Rangers — {2}{G} 2/2 that only fliers can stop.
pub fn treetop_rangers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Flying)))],
        ..creature(
            "Treetop Rangers",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Ranger],
            2,
            2,
        )
    }
}

/// Treefolk Seedlings — {2}{G} 2/* whose toughness is your Forest count.
pub fn treefolk_seedlings() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatchingToughness {
            base_p: 2,
            base_t: 0,
            filter: Box::new(R::HasLandType(LandType::Forest)),
        }),
        ..creature("Treefolk Seedlings", cost(&[generic(2), g()]), vec![CreatureType::Treefolk], 2, 0)
    }
}

/// Serra Avatar — {4}{W}{W}{W} whose body is your life total; it reshuffles
/// itself rather than staying dead.
pub fn serra_avatar() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ControllerLife),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::ShuffleSelfIntoLibrary,
        }],
        ..creature(
            "Serra Avatar",
            cost(&[generic(4), w(), w(), w()]),
            vec![CreatureType::Avatar],
            0,
            0,
        )
    }
}

/// Unworthy Dead — {1}{B} 1/1 that regenerates for {B}.
pub fn unworthy_dead() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![regenerate_for(cost(&[b()]))],
        ..creature(
            "Unworthy Dead",
            cost(&[generic(1), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Skeleton],
            1,
            1,
        )
    }
}

/// Silent Attendant — {2}{W} 0/2 that taps for a life.
pub fn silent_attendant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Silent Attendant",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            0,
            2,
        )
    }
}

/// Shimmering Barrier — {1}{W} 1/3 Wall with first strike and cycling.
pub fn shimmering_barrier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::FirstStrike, cycling_two()],
        ..creature("Shimmering Barrier", cost(&[generic(1), w()]), vec![CreatureType::Wall], 1, 3)
    }
}

/// Spined Fluke — {2}{B} 5/1 that eats a friend on arrival and regenerates.
pub fn spined_fluke() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::You,
            count: Value::ONE,
            filter: R::Creature,
        })],
        activated_abilities: vec![regenerate_for(cost(&[b()]))],
        ..creature(
            "Spined Fluke",
            cost(&[generic(2), b()]),
            vec![CreatureType::Worm, CreatureType::Horror],
            5,
            1,
        )
    }
}

/// Skirge Familiar — {4}{B} 3/2 flier that turns cards into black mana.
pub fn skirge_familiar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black]),
            },
            ..Default::default()
        }],
        ..creature(
            "Skirge Familiar",
            cost(&[generic(4), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Imp],
            3,
            2,
        )
    }
}

/// Reclusive Wight — {3}{B} 4/4 that can't stand company.
pub fn reclusive_wight() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Nonland.and(R::OtherThanSource),
            })),
            effect: Effect::SacrificeSource,
        }],
        ..creature(
            "Reclusive Wight",
            cost(&[generic(3), b()]),
            vec![CreatureType::Zombie, CreatureType::Minion],
            4,
            4,
        )
    }
}

/// Imaginary Pet — {1}{U} 4/4 that runs home while you hold cards.
pub fn imaginary_pet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::ValueAtLeast(
                Value::HandSizeOf(PlayerRef::You),
                Value::ONE,
            )),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..creature("Imaginary Pet", cost(&[generic(1), u()]), vec![CreatureType::Illusion], 4, 4)
    }
}

/// Flesh Reaver — {1}{B} 4/4 that bites you back.
pub fn flesh_reaver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature(
            "Flesh Reaver",
            cost(&[generic(1), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            4,
            4,
        )
    }
}

/// Dromosaur — {2}{R} 2/3 that trades toughness for punch in combat.
pub fn dromosaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: [EventKind::Blocks, EventKind::BecomesBlocked]
            .map(|kind| TriggeredAbility {
                event: EventSpec::new(kind, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                },
            })
            .to_vec(),
        ..creature("Dromosaur", cost(&[generic(2), r()]), vec![CreatureType::Dinosaur], 2, 3)
    }
}

/// Cave Tiger — {2}{G} 2/2 that grows when something stops it.
pub fn cave_tiger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Cave Tiger", cost(&[generic(2), g()]), vec![CreatureType::Cat], 2, 2)
    }
}

/// Viashino Weaponsmith — {3}{R} 2/2 that swells when blocked.
pub fn viashino_weaponsmith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Viashino Weaponsmith", cost(&[generic(3), r()]), vec![CreatureType::Lizard], 2, 2)
    }
}

/// Titania's Chosen — {2}{G} 1/1 that grows off every green spell.
pub fn titanias_chosen() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Green),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Titania's Chosen",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Archer],
            1,
            1,
        )
    }
}

/// Stern Proctor — {U}{U} 1/2 that bounces an artifact or enchantment.
pub fn stern_proctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Artifact.or(R::Enchantment)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..creature(
            "Stern Proctor",
            cost(&[u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Wizard Mentor — {2}{U} 2/2 that packs itself and a friend up.
pub fn wizard_mentor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Wizard Mentor",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Abyssal Horror — {4}{B}{B} 2/2 flier that strips two cards on arrival.
pub fn abyssal_horror() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Discard {
            who: target_filtered(R::Player),
            amount: Value::Const(2),
            random: false,
        })],
        ..creature("Abyssal Horror", cost(&[generic(4), b(), b()]), vec![CreatureType::Horror], 2, 2)
    }
}

/// Dark Hatchling — {4}{B}{B} 3/3 flier whose arrival kills a nonblack body.
pub fn dark_hatchling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DestroyNoRegen {
            what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
        })],
        ..creature("Dark Hatchling", cost(&[generic(4), b(), b()]), vec![CreatureType::Horror], 3, 3)
    }
}

fn paladin(name: &'static str, hated: Color) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasColor(hated))),
            },
            ..Default::default()
        }],
        ..creature(
            name,
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Eastern Paladin — {2}{B}{B} 3/3 that snipes green.
pub fn eastern_paladin() -> CardDefinition {
    paladin("Eastern Paladin", Color::Green)
}

/// Western Paladin — {2}{B}{B} 3/3 that snipes white.
pub fn western_paladin() -> CardDefinition {
    paladin("Western Paladin", Color::White)
}

/// Intrepid Hero — {2}{W} 1/1 that taps to kill anything big.
pub fn intrepid_hero() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::PowerAtLeast(4))),
            },
            ..Default::default()
        }],
        ..creature(
            "Intrepid Hero",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Elite Archers — {5}{W} 3/3 that shoots anything in combat.
pub fn elite_archers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..creature(
            "Elite Archers",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Archer],
            3,
            3,
        )
    }
}

/// Shivan Hellkite — {5}{R}{R} 5/5 flier with a machine-gun ability.
pub fn shivan_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Shivan Hellkite", cost(&[generic(5), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Child of Gaea — {3}{G}{G}{G} 7/7 trampler with an upkeep tax and regeneration.
pub fn child_of_gaea() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![upkeep_sacrifice_unless_pay(cost(&[g(), g()]))],
        activated_abilities: vec![regenerate_for(cost(&[generic(1), g()]))],
        ..creature(
            "Child of Gaea",
            cost(&[generic(3), g(), g(), g()]),
            vec![CreatureType::Elemental],
            7,
            7,
        )
    }
}

/// Drifting Djinn — {4}{U}{U} 5/5 flier with an upkeep tax and cycling.
pub fn drifting_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, cycling_two()],
        triggered_abilities: vec![upkeep_sacrifice_unless_pay(cost(&[generic(1), u()]))],
        ..creature("Drifting Djinn", cost(&[generic(4), u(), u()]), vec![CreatureType::Djinn], 5, 5)
    }
}

/// Endless Wurm — {3}{G}{G} 9/9 trampler that eats an enchantment each upkeep.
pub fn endless_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::You,
                cost: WardCost::SacrificeMatching(Box::new(R::Enchantment)),
                then: Box::new(Effect::SacrificeSource),
            },
        }],
        ..creature("Endless Wurm", cost(&[generic(3), g(), g()]), vec![CreatureType::Wurm], 9, 9)
    }
}

/// Citanul Hierophants — {3}{G} 3/2 that turns your team into Llanowar Elves.
pub fn citanul_hierophants() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have \"{T}: Add {G}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                ability: tap_add(Color::Green),
                condition: None,
            },
        }],
        ..creature(
            "Citanul Hierophants",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            3,
            2,
        )
    }
}

/// Morphling — {3}{U}{U} 3/3 with the full toolbox.
pub fn morphling() -> CardDefinition {
    let pump = |p: i32, t: i32| ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(p),
            toughness: Value::Const(t),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    let grant = |kw: Keyword| ActivatedAbility {
        mana_cost: cost(&[u()]),
        effect: Effect::GrantKeyword {
            what: Selector::This,
            keyword: kw,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
            grant(Keyword::Flying),
            grant(Keyword::Shroud),
            pump(1, -1),
            pump(-1, 1),
        ],
        ..creature(
            "Morphling",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Shapeshifter],
            3,
            3,
        )
    }
}

/// Mana Leech — {2}{B} 1/1 that holds a land down while it stays tapped.
pub fn mana_leech() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TapAndUntapLock { what: target_filtered(R::Land) },
            ..Default::default()
        }],
        ..creature("Mana Leech", cost(&[generic(2), b()]), vec![CreatureType::Leech], 1, 1)
    }
}

/// Somnophore — {2}{U}{U} 2/2 flier that pins a creature per hit.
pub fn somnophore() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourSourceDamagedOpponent),
            effect: Effect::TapAndLockWhileSourcePresent {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
        }],
        ..creature("Somnophore", cost(&[generic(2), u(), u()]), vec![CreatureType::Illusion], 2, 2)
    }
}

/// Phyrexian Colossus — {7} 8/8 that pays life to swing and needs a crowd to stop.
pub fn phyrexian_colossus() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::CantBeBlockedExceptByN(3)],
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 8,
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..creature(
            "Phyrexian Colossus",
            cost(&[generic(7)]),
            vec![CreatureType::Phyrexian, CreatureType::Golem],
            8,
            8,
        )
    }
}

/// Wirecat — {4} 4/3 that sulks while any enchantment is out.
pub fn wirecat() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: [Keyword::CantAttack, Keyword::CantBlock]
            .map(|keyword| StaticAbility {
                description: "This creature can't attack or block if an enchantment is on the battlefield.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::SelectorExists(Selector::EachPermanent(R::Enchantment)),
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: Selector::This,
                        keyword,
                    }),
                },
            })
            .to_vec(),
        ..creature("Wirecat", cost(&[generic(4)]), vec![CreatureType::Cat], 4, 3)
    }
}

/// Faith Healer — {1}{W} 1/1 that cashes enchantments in for life.
pub fn faith_healer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Enchantment, 1)),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::SacrificedManaValue,
            },
            ..Default::default()
        }],
        ..creature(
            "Faith Healer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Sleeper Agent — {B} 3/3 you hand to an opponent, then it bleeds you.
pub fn sleeper_agent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::Target(0)),
                duration: Duration::Permanent,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::DealDamage { to: Selector::You, amount: Value::Const(2) },
            },
        ],
        ..creature(
            "Sleeper Agent",
            cost(&[b()]),
            vec![CreatureType::Phyrexian, CreatureType::Minion],
            3,
            3,
        )
    }
}

/// Scoria Wurm — {4}{R} 7/7 that may pack itself up each upkeep.
pub fn scoria_wurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Noop),
                on_tails: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                }),
            },
        }],
        ..creature("Scoria Wurm", cost(&[generic(4), r()]), vec![CreatureType::Wurm], 7, 7)
    }
}

/// Retromancer — {2}{R}{R} 3/3 that burns whoever points at it.
pub fn retromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(3),
            },
        }],
        ..creature(
            "Retromancer",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Lizard, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// Witch Engine — {5}{B} 4/4 swampwalker that sells its mana to an opponent.
pub fn witch_engine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Black, Value::Const(4)),
                },
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::Target(0)),
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Witch Engine", cost(&[generic(5), b()]), vec![CreatureType::Horror], 4, 4)
    }
}

/// Vebulid — {B} 0/0 that grows each upkeep and dies if it fights.
pub fn vebulid() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::MayDo {
                    description: "Put a +1/+1 counter on Vebulid".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::AtEndOfCombat {
                    body: Box::new(Effect::Destroy { what: Selector::This }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: Effect::AtEndOfCombat {
                    body: Box::new(Effect::Destroy { what: Selector::This }),
                },
            },
        ],
        ..creature(
            "Vebulid",
            cost(&[b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            0,
            0,
        )
    }
}

/// Songstitcher — {W} 1/1 that blanks a flying attacker's damage.
pub fn songstitcher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::PreventCombatDamageByTargetThisTurn {
                target: target_filtered(
                    R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Flying)),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Songstitcher",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Claws of Gix — {0}. Grind anything into a life point.
pub fn claws_of_gix() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Permanent, 1)),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Claws of Gix", cost(&[]))
    }
}

/// Dragon Blood — {3}. A counter a turn, for a price.
pub fn dragon_blood() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..artifact("Dragon Blood", cost(&[generic(3)]))
    }
}

/// Whetstone — {3}. Everyone mills two.
pub fn whetstone() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..artifact("Whetstone", cost(&[generic(3)]))
    }
}

/// Crystal Chimes — {3}. Buy back every enchantment in your graveyard.
pub fn crystal_chimes() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::ReturnGraveyardCardsToHand {
                filter: R::Enchantment,
                max: Value::Const(99),
            },
            ..Default::default()
        }],
        ..artifact("Crystal Chimes", cost(&[generic(3)]))
    }
}

/// Citanul Flute — {5}. Tutor a creature by size.
pub fn citanul_flute() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature.and(R::ManaValueAtMostXFromCost),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..artifact("Citanul Flute", cost(&[generic(5)]))
    }
}

/// Chimeric Staff — {4}. An X/X for as much as you can pay.
pub fn chimeric_staff() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::XFromCost,
                toughness: Value::XFromCost,
                creature_types: vec![CreatureType::Construct],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Chimeric Staff", cost(&[generic(4)]))
    }
}

/// Mishra's Helix — {5}. Lock down X lands.
pub fn mishras_helix() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::TapUpToValue {
                count: Value::XFromCost,
                filter: R::Land,
                skip_untap: false,
                exact: true,
            },
            ..Default::default()
        }],
        ..artifact("Mishra's Helix", cost(&[generic(5)]))
    }
}

/// Barrin's Codex — {4}. Bank pages, cash them in for cards.
pub fn barrins_codex() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Put a page counter on Barrin's Codex".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Page,
                    amount: Value::ONE,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Page,
                },
            },
            ..Default::default()
        }],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Book],
            ..Default::default()
        },
        ..artifact("Barrin's Codex", cost(&[generic(4)]))
    }
}

/// Lotus Blossom — {2}. Petals now, a Lotus later.
pub fn lotus_blossom() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Put a petal counter on Lotus Blossom".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Petal,
                    amount: Value::ONE,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Petal,
                }),
            },
            ..Default::default()
        }],
        ..artifact("Lotus Blossom", cost(&[generic(2)]))
    }
}



/// Grafted Skullcap — {4}. An extra card a turn, at the cost of your hand.
pub fn grafted_skullcap() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Draw),
                    EventScope::YourControl,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::YourControl,
                ),
                effect: Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
            },
        ],
        ..artifact("Grafted Skullcap", cost(&[generic(4)]))
    }
}


// ── Enchantments ────────────────────────────────────────────────────────────

fn all_creatures_have(name: &'static str, description: &'static str, keyword: Keyword) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description,
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword,
            },
        }],
        ..enchantment(name, cost(&[generic(1), w()]))
    }
}

/// Absolute Grace — every creature shrugs off black.
pub fn absolute_grace() -> CardDefinition {
    all_creatures_have(
        "Absolute Grace",
        "All creatures have protection from black.",
        Keyword::Protection(Color::Black),
    )
}

/// Absolute Law — every creature shrugs off red.
pub fn absolute_law() -> CardDefinition {
    all_creatures_have(
        "Absolute Law",
        "All creatures have protection from red.",
        Keyword::Protection(Color::Red),
    )
}

/// Bedlam — {2}{R}{R}. Nobody blocks.
pub fn bedlam() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures can't block.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::CantBlock,
            },
        }],
        ..enchantment("Bedlam", cost(&[generic(2), r(), r()]))
    }
}

/// Crosswinds — {1}{G}. Fliers lose their edge.
pub fn crosswinds() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures with flying get -2/-0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasKeyword(Keyword::Flying)),
                ),
                power: -2,
                toughness: 0,
            },
        }],
        ..enchantment("Crosswinds", cost(&[generic(1), g()]))
    }
}


/// Oppression — {1}{B}{B}. Every spell costs a card.
pub fn oppression() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..enchantment("Oppression", cost(&[generic(1), b(), b()]))
    }
}

/// Planar Void — {B}. Nothing rests in a graveyard.
pub fn planar_void() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Whenever another card is put into a graveyard from anywhere, exile it.",
            effect: StaticEffect::ExileCardsBoundForGraveyard {
                opponents_only: false,
                own_only: false,
                colors: None,
                card_types: None,
                void_counter: false,
            },
        }],
        ..enchantment("Planar Void", cost(&[b()]))
    }
}

/// Bereavement — {1}{B}. Green deaths cost their controller a card.
pub fn bereavement() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasColor(Color::Green)),
                }),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..enchantment("Bereavement", cost(&[generic(1), b()]))
    }
}

/// Yawgmoth's Edict — {1}{B}. White spells bleed their caster.
pub fn yawgmoths_edict() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::White),
                }),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(
                        Selector::TriggerSource,
                    ))),
                    amount: Value::ONE,
                },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..enchantment("Yawgmoth's Edict", cost(&[generic(1), b()]))
    }
}

/// Scald — {1}{R}. Islands burn their user.
pub fn scald() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasLandType(LandType::Island),
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..enchantment("Scald", cost(&[generic(1), r()]))
    }
}


// ── Verse-counter cycle ─────────────────────────────────────────────────────

/// Torch Song — {2}{R}. Verses, then a burn spell.
pub fn torch_song() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: verses() },
            ..Default::default()
        }],
        ..enchantment("Torch Song", cost(&[generic(2), r()]))
    }
}

/// War Dance — {G}. Verses, then a pump.
pub fn war_dance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: verses(),
                toughness: verses(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment("War Dance", cost(&[g()]))
    }
}

/// Midsummer Revel — {3}{G}{G}. Verses, then a herd of Beasts.
pub fn midsummer_revel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: verses(),
                definition: crate::card::TokenDefinition {
                    name: "Beast".into(),
                    power: 3,
                    toughness: 3,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Beast],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..enchantment("Midsummer Revel", cost(&[generic(3), g(), g()]))
    }
}

/// Rumbling Crescendo — {3}{R}{R}. Verses, then a land wipe.
pub fn rumbling_crescendo() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::CapTargetsAt {
                amount: verses(),
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Land,
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
            ..Default::default()
        }],
        ..enchantment("Rumbling Crescendo", cost(&[generic(3), r(), r()]))
    }
}

/// Serra's Liturgy — {2}{W}{W}. Verses, then artifact/enchantment removal.
pub fn serras_liturgy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_cost: true,
            effect: Effect::CapTargetsAt {
                amount: verses(),
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Artifact.or(R::Enchantment),
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
            ..Default::default()
        }],
        ..enchantment("Serra's Liturgy", cost(&[generic(2), w(), w()]))
    }
}

/// Vile Requiem — {2}{B}{B}. Verses, then unregenerable removal.
pub fn vile_requiem() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            effect: Effect::CapTargetsAt {
                amount: verses(),
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                    effect: Box::new(Effect::DestroyNoRegen { what: Selector::Target(0) }),
                }),
            },
            ..Default::default()
        }],
        ..enchantment("Vile Requiem", cost(&[generic(2), b(), b()]))
    }
}

/// Recantation — {3}{U}{U}. Verses, then a mass bounce.
pub fn recantation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CapTargetsAt {
                amount: verses(),
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Permanent,
                    effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
                }),
            },
            ..Default::default()
        }],
        ..enchantment("Recantation", cost(&[generic(3), u(), u()]))
    }
}

/// Lilting Refrain — {1}{U}. Verses, then a scaling Force Spike.
pub fn lilting_refrain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: Selector::Target(0),
                mana_cost: cost(&[]),
                exile: false,
                extra_generic: Some(verses()),
            },
            ..Default::default()
        }],
        ..enchantment("Lilting Refrain", cost(&[generic(1), u()]))
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Bravado — {1}{R}. +1/+1 for each of your other creatures.
pub fn bravado() -> CardDefinition {
    creature_aura(
        "Bravado",
        cost(&[generic(1), r()]),
        EquipBonus {
            scale: Some(EquipScale {
                filter: R::Creature,
                per_power: 1,
                per_toughness: 1,
                exclude_host: true,
                ..Default::default()
            }),
            ..Default::default()
        },
    )
}

/// Brilliant Halo — {1}{W}. +1/+2, and it comes back.
pub fn brilliant_halo() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![returns_to_hand()],
        ..creature_aura(
            "Brilliant Halo",
            cost(&[generic(1), w()]),
            EquipBonus { power: 1, toughness: 2, ..Default::default() },
        )
    }
}

/// Despondency — {1}{B}. -2/-0, and it comes back.
pub fn despondency() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![returns_to_hand()],
        ..creature_aura(
            "Despondency",
            cost(&[generic(1), b()]),
            EquipBonus { power: -2, toughness: 0, ..Default::default() },
        )
    }
}

/// Launch — {1}{U}. Flying, and it comes back.
pub fn launch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![returns_to_hand()],
        ..creature_aura(
            "Launch",
            cost(&[generic(1), u()]),
            EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() },
        )
    }
}

/// Cloak of Mists — {1}{U}. The host walks past everything.
pub fn cloak_of_mists() -> CardDefinition {
    creature_aura(
        "Cloak of Mists",
        cost(&[generic(1), u()]),
        EquipBonus { keywords: vec![Keyword::Unblockable], ..Default::default() },
    )
}

/// Reflexes — {R}. First strike.
pub fn reflexes() -> CardDefinition {
    creature_aura(
        "Reflexes",
        cost(&[r()]),
        EquipBonus { keywords: vec![Keyword::FirstStrike], ..Default::default() },
    )
}

/// Sicken — {B}. -1/-1 with cycling.
pub fn sicken() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..creature_aura(
            "Sicken",
            cost(&[b()]),
            EquipBonus { power: -1, toughness: -1, ..Default::default() },
        )
    }
}

/// Serra's Embrace — {2}{W}{W}. +2/+2, flying and vigilance.
pub fn serras_embrace() -> CardDefinition {
    creature_aura(
        "Serra's Embrace",
        cost(&[generic(2), w(), w()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying, Keyword::Vigilance],
            ..Default::default()
        },
    )
}

/// Vampiric Embrace — {2}{B}{B}. +2/+2 and flying; its victims feed it.
pub fn vampiric_embrace() -> CardDefinition {
    creature_aura(
        "Vampiric Embrace",
        cost(&[generic(2), b(), b()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        },
    )
}

/// Zephid's Embrace — {2}{U}{U}. +2/+2, flying and shroud.
pub fn zephids_embrace() -> CardDefinition {
    creature_aura(
        "Zephid's Embrace",
        cost(&[generic(2), u(), u()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying, Keyword::Shroud],
            ..Default::default()
        },
    )
}

/// Gaea's Embrace — {2}{G}{G}. +3/+3, trample, and regeneration.
pub fn gaeas_embrace() -> CardDefinition {
    creature_aura(
        "Gaea's Embrace",
        cost(&[generic(2), g(), g()]),
        EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Trample],
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Shiv's Embrace — {2}{R}{R}. +2/+2, flying, and firebreathing.
pub fn shivs_embrace() -> CardDefinition {
    creature_aura(
        "Shiv's Embrace",
        cost(&[generic(2), r(), r()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying],
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Hermetic Study — {1}{U}. The host becomes a pinger.
pub fn hermetic_study() -> CardDefinition {
    creature_aura(
        "Hermetic Study",
        cost(&[generic(1), u()]),
        EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Fiery Mantle — {1}{R}. Firebreathing that comes back.
pub fn fiery_mantle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![returns_to_hand()],
        ..creature_aura(
            "Fiery Mantle",
            cost(&[generic(1), r()]),
            EquipBonus {
                activated_abilities: vec![ActivatedAbility {
                    mana_cost: cost(&[r()]),
                    effect: Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
    }
}

/// Fortitude — {1}{G}. Forests regenerate the host; it comes back.
pub fn fortitude() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![returns_to_hand()],
        ..creature_aura(
            "Fortitude",
            cost(&[generic(1), g()]),
            EquipBonus {
                activated_abilities: vec![ActivatedAbility {
                    sac_other_filter: Some((R::HasLandType(LandType::Forest), 1)),
                    effect: Effect::Regenerate { what: Selector::This },
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
    }
}

/// Pariah — {2}{W}. Your damage lands on the host instead.
pub fn pariah() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All damage that would be dealt to you is dealt to enchanted creature instead.",
            effect: StaticEffect::RedirectControllerDamageToEquippedCreature,
        }],
        ..creature_aura("Pariah", cost(&[generic(2), w()]), EquipBonus::default())
    }
}


/// Parasitic Bond — {3}{B}. The host's controller bleeds each upkeep.
pub fn parasitic_bond() -> CardDefinition {
    creature_aura(
        "Parasitic Bond",
        cost(&[generic(3), b()]),
        EquipBonus {
            triggers_on_equipment: true,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::IsTurnOf(PlayerRef::ControllerOf(Box::new(
                    Selector::AttachedTo(Box::new(Selector::This)),
                )))),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::AttachedTo(
                        Box::new(Selector::This),
                    )))),
                    amount: Value::Const(2),
                },
            }],
            ..Default::default()
        },
    )
}

/// Destructive Urge — {1}{R}{R}. Connecting costs the defender a land.
pub fn destructive_urge() -> CardDefinition {
    creature_aura(
        "Destructive Urge",
        cost(&[generic(1), r(), r()]),
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Target(0)),
                    count: Value::ONE,
                    filter: R::Land,
                },
            }],
            ..Default::default()
        },
    )
}

/// Venomous Fangs — {2}{G}. The host's bite is lethal.
pub fn venomous_fangs() -> CardDefinition {
    creature_aura(
        "Venomous Fangs",
        cost(&[generic(2), g()]),
        EquipBonus { keywords: vec![Keyword::Deathtouch], ..Default::default() },
    )
}

/// Fertile Ground — {1}{G}. The enchanted land yields an extra colour.
pub fn fertile_ground() -> CardDefinition {
    CardDefinition {
        name: "Fertile Ground",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![StaticAbility {
            description: "Whenever enchanted land is tapped for mana, add one mana of any color.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: true,
                filter: R::Land,
                extra: crate::effect::ExtraManaKind::AnyColor,
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}

/// Lingering Mirage — {1}{U}. The enchanted land is an Island; cycling too.
pub fn lingering_mirage() -> CardDefinition {
    CardDefinition {
        name: "Lingering Mirage",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![cycling_two()],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus {
            set_land_types: Some(vec![LandType::Island]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Acidic Soil — {2}{R}. Everyone's own lands burn them.
pub fn acidic_soil() -> CardDefinition {
    sorcery(
        "Acidic Soil",
        cost(&[generic(2), r()]),
        Effect::DealDamageToEachPlayerPerPermanent {
            filter: R::Land,
            amount: Value::ONE,
            flat: false,
        },
    )
}

/// Disorder — {1}{R}. Two to every white creature and their controllers.
pub fn disorder() -> CardDefinition {
    sorcery(
        "Disorder",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            // The player half reads the board first — CR 608.2 resolves the
            // whole spell at once, so a creature dying to the sweep must not
            // spare its controller.
            Effect::DealDamageToEachPlayerPerPermanent {
                filter: R::Creature.and(R::HasColor(Color::White)),
                amount: Value::Const(2),
                flat: true,
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::White))),
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Falter — {1}{R}. The ground can't block this turn.
pub fn falter() -> CardDefinition {
    instant(
        "Falter",
        cost(&[generic(1), r()]),
        Effect::GrantKeywordToMatchingThisTurn {
            filter: R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
            keyword: Keyword::CantBlock,
        },
    )
}

/// Curfew — {U}. Everyone packs one creature away.
pub fn curfew() -> CardDefinition {
    instant(
        "Curfew",
        cost(&[u()]),
        Effect::EachPlayerReturnsAMatchingPermanent { filter: R::Creature },
    )
}

/// Steam Blast — {2}{R}. Two to everything with a life total or a body.
pub fn steam_blast() -> CardDefinition {
    sorcery(
        "Steam Blast",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Unnerve — {3}{B}. Two cards off each opponent.
pub fn unnerve() -> CardDefinition {
    sorcery(
        "Unnerve",
        cost(&[generic(3), b()]),
        Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
            random: false,
        },
    )
}

/// Symbiosis — {1}{G}. +2/+2 for two.
pub fn symbiosis() -> CardDefinition {
    instant(
        "Symbiosis",
        cost(&[generic(1), g()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature,
            effect: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Titania's Boon — {3}{G}. A counter on each of your creatures.
pub fn titanias_boon() -> CardDefinition {
    sorcery(
        "Titania's Boon",
        cost(&[generic(3), g()]),
        Effect::AddCounter {
            what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
    )
}

/// Shower of Sparks — {R}. One each to a creature and a player.
pub fn shower_of_sparks() -> CardDefinition {
    instant(
        "Shower of Sparks",
        cost(&[r()]),
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Player.or(R::Planeswalker),
                },
                amount: Value::ONE,
            },
        ]),
    )
}

/// Sunder — {3}{U}{U}. Every land goes home.
pub fn sunder() -> CardDefinition {
    instant(
        "Sunder",
        cost(&[generic(3), u(), u()]),
        Effect::Move {
            what: Selector::EachPermanent(R::Land),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Lull — {1}{G}. Combat damage is off this turn; or a card.
pub fn lull() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..instant("Lull", cost(&[generic(1), g()]), Effect::PreventAllCombatDamageThisTurn)
    }
}


/// Reprocess — {2}{B}{B}. Trade permanents for cards.
pub fn reprocess() -> CardDefinition {
    sorcery(
        "Reprocess",
        cost(&[generic(2), b(), b()]),
        Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Artifact.or(R::Creature).or(R::Land),
            per_each: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        },
    )
}




/// Gamble — {R}. Any card you like, minus a random one.
pub fn gamble() -> CardDefinition {
    sorcery(
        "Gamble",
        cost(&[r()]),
        Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
        ]),
    )
}

// ── The Opal / Hidden / Veiled animation cycles (CR 205.1b) ─────────────────

/// "When an opponent [event], if this permanent is an enchantment, it becomes
/// a P/T creature." The animation replaces the enchantment type outright.
fn stands_up(
    kind: EventKind,
    trigger_filter: Option<R>,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> TriggeredAbility {
    let mut event = EventSpec::new(kind, EventScope::OpponentControl).with_filter(
        Predicate::EntityMatches { what: Selector::This, filter: R::Enchantment },
    );
    if let Some(f) = trigger_filter {
        event = event.with_filter(Predicate::All(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::Enchantment },
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: f },
        ]));
    }
    TriggeredAbility {
        event,
        effect: Effect::BecomeCreatureLosingTypes {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            creature_types: types,
            keywords,
        },
    }
}

fn opal(
    name: &'static str,
    c: crate::mana::ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![stands_up(
            EventKind::SpellCast,
            Some(R::Creature),
            power,
            toughness,
            types,
            keywords,
        )],
        ..enchantment(name, c)
    }
}

/// Opal Caryatid — {W}. A 2/2 Soldier when an opponent casts a creature.
pub fn opal_caryatid() -> CardDefinition {
    opal("Opal Caryatid", cost(&[w()]), 2, 2, vec![CreatureType::Soldier], vec![])
}

/// Opal Gargoyle — {1}{W}. A 2/2 flier when an opponent casts a creature.
pub fn opal_gargoyle() -> CardDefinition {
    opal(
        "Opal Gargoyle",
        cost(&[generic(1), w()]),
        2,
        2,
        vec![CreatureType::Gargoyle],
        vec![Keyword::Flying],
    )
}

/// Opal Archangel — {4}{W}. A 5/5 Angel when an opponent casts a creature.
pub fn opal_archangel() -> CardDefinition {
    opal(
        "Opal Archangel",
        cost(&[generic(4), w()]),
        5,
        5,
        vec![CreatureType::Angel],
        vec![Keyword::Flying, Keyword::Vigilance],
    )
}

/// Opal Titan — {2}{W}{W}. A 4/4 Giant when an opponent casts a creature.
/// The printed protection from that spell's colours is dropped.
pub fn opal_titan() -> CardDefinition {
    opal("Opal Titan", cost(&[generic(2), w(), w()]), 4, 4, vec![CreatureType::Giant], vec![])
}

/// Opal Acrolith — {2}{W}. A 2/4 Soldier when an opponent casts a creature;
/// `{0}` turns it back into an enchantment.
pub fn opal_acrolith() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::SetCardTypesTo {
                what: Selector::This,
                card_types: vec![CardType::Enchantment],
            },
            ..Default::default()
        }],
        ..opal("Opal Acrolith", cost(&[generic(2), w()]), 2, 4, vec![CreatureType::Soldier], vec![])
    }
}

/// Hidden Ancients — {1}{G}. A 5/5 Treefolk when an opponent casts an
/// enchantment.
pub fn hidden_ancients() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![stands_up(
            EventKind::SpellCast,
            Some(R::Enchantment),
            5,
            5,
            vec![CreatureType::Treefolk],
            vec![],
        )],
        ..enchantment("Hidden Ancients", cost(&[generic(1), g()]))
    }
}

/// Hidden Guerrillas — {G}. A 5/3 trampler when an opponent casts an artifact.
pub fn hidden_guerrillas() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![stands_up(
            EventKind::SpellCast,
            Some(R::Artifact),
            5,
            3,
            vec![CreatureType::Soldier],
            vec![Keyword::Trample],
        )],
        ..enchantment("Hidden Guerrillas", cost(&[g()]))
    }
}

/// Hidden Spider — {G}. A 3/5 reach blocker when an opponent casts a flier.
pub fn hidden_spider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![stands_up(
            EventKind::SpellCast,
            Some(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            3,
            5,
            vec![CreatureType::Spider],
            vec![Keyword::Reach],
        )],
        ..enchantment("Hidden Spider", cost(&[g()]))
    }
}

/// Hidden Herd — {G}. A 3/3 Beast when an opponent plays a nonbasic land.
pub fn hidden_herd() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![stands_up(
            EventKind::LandPlayed,
            Some(R::IsNonbasicLand),
            3,
            3,
            vec![CreatureType::Beast],
            vec![],
        )],
        ..enchantment("Hidden Herd", cost(&[g()]))
    }
}

/// Hidden Stag — {1}{G}. A 3/2 when an opponent plays a land; your own land
/// drop settles it back down.
pub fn hidden_stag() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            stands_up(
                EventKind::LandPlayed,
                None,
                3,
                2,
                vec![CreatureType::Elk, CreatureType::Beast],
                vec![],
            ),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::This, filter: R::Creature },
                ),
                effect: Effect::SetCardTypesTo {
                    what: Selector::This,
                    card_types: vec![CardType::Enchantment],
                },
            },
        ],
        ..enchantment("Hidden Stag", cost(&[generic(1), g()]))
    }
}

/// Veil of Birds — {U}. A 1/1 flier when an opponent casts anything.
pub fn veil_of_birds() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![stands_up(
            EventKind::SpellCast,
            None,
            1,
            1,
            vec![CreatureType::Bird],
            vec![Keyword::Flying],
        )],
        ..enchantment("Veil of Birds", cost(&[u()]))
    }
}

/// Veiled Serpent — {2}{U}. A 4/4 Serpent that needs an Island to swing; or a
/// card.
pub fn veiled_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        triggered_abilities: vec![stands_up(
            EventKind::SpellCast,
            None,
            4,
            4,
            vec![CreatureType::Serpent],
            vec![Keyword::CantAttackUnlessLandTypeOnBattlefield(LandType::Island)],
        )],
        ..enchantment("Veiled Serpent", cost(&[generic(2), u()]))
    }
}

/// Veiled Sentry — {U}. An Illusion the size of the spell that woke it.
pub fn veiled_sentry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::This, filter: R::Enchantment },
            ),
            effect: Effect::BecomeCreatureLosingTypes {
                what: Selector::This,
                power: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                toughness: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                creature_types: vec![CreatureType::Illusion],
                keywords: vec![],
            },
        }],
        ..enchantment("Veiled Sentry", cost(&[u()]))
    }
}

/// Lurking Evil — {B}{B}{B}. Half your life stands it up as a 4/4 flier.
pub fn lurking_evil() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            half_life_cost: true,
            effect: Effect::BecomeCreatureLosingTypes {
                what: Selector::This,
                power: Value::Const(4),
                toughness: Value::Const(4),
                creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
                keywords: vec![Keyword::Flying],
            },
            ..Default::default()
        }],
        ..enchantment("Lurking Evil", cost(&[b(), b(), b()]))
    }
}
