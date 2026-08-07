//! Weatherlight (WTH) closing waves. Tests in `classic_sets/wth`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, StaticAbility,
    Subtypes, TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{deal, draw, etb, etb_draw, etb_loot, target_filtered};
use crate::effect::{
    Duration, Effect, Predicate, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Sorcery], ..instant(name, c, effect) }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

fn cu(c: CumulativeUpkeepCost) -> Keyword {
    Keyword::CumulativeUpkeep(c)
}

// ── Cumulative-upkeep creatures ─────────────────────────────────────────────

/// Arctic Wolves — {3}{G}{G} 4/5. Cumulative upkeep {2}, replaces itself.
pub fn arctic_wolves() -> CardDefinition {
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![etb_draw(1)],
        ..creature("Arctic Wolves", cost(&[generic(3), g(), g()]), vec![CreatureType::Wolf], 4, 5)
    }
}

/// Gallowbraid — {3}{B}{B} 5/5 legendary trampler on a life clock.
pub fn gallowbraid() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Trample, cu(CumulativeUpkeepCost::Life(1))],
        ..creature(
            "Gallowbraid",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            5,
            5,
        )
    }
}

/// Morinfen — {3}{B}{B} 5/4 legendary flier on a life clock.
pub fn morinfen() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Flying, cu(CumulativeUpkeepCost::Life(1))],
        ..creature(
            "Morinfen",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            5,
            4,
        )
    }
}

/// Uktabi Efreet — {2}{G}{G} 5/4 with cumulative upkeep {G}.
pub fn uktabi_efreet() -> CardDefinition {
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::Mana(cost(&[g()])))],
        ..creature("Uktabi Efreet", cost(&[generic(2), g(), g()]), vec![CreatureType::Efreet], 5, 4)
    }
}

/// Volunteer Reserves — {1}{W} 2/4 bander with cumulative upkeep {1}.
pub fn volunteer_reserves() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Banding, cu(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        ..creature(
            "Volunteer Reserves",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            4,
        )
    }
}

/// Revered Unicorn — {1}{W} 2/3. Cumulative upkeep {1}; the age counters pay
/// out as life when it leaves.
pub fn revered_unicorn() -> CardDefinition {
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Age },
            },
        }],
        ..creature("Revered Unicorn", cost(&[generic(1), w()]), vec![CreatureType::Unicorn], 2, 3)
    }
}

// ── Vanilla-shaped creatures ────────────────────────────────────────────────

/// Benalish Infantry — {2}{W} 1/3 bander.
pub fn benalish_infantry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Banding],
        ..creature(
            "Benalish Infantry",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            3,
        )
    }
}

/// Razortooth Rats — {2}{B} 2/1 with fear.
pub fn razortooth_rats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        ..creature("Razortooth Rats", cost(&[generic(2), b()]), vec![CreatureType::Rat], 2, 1)
    }
}

/// Shadow Rider — {2}{B}{B} 3/3 with flanking.
pub fn shadow_rider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        ..creature("Shadow Rider", cost(&[generic(2), b(), b()]), vec![CreatureType::Knight], 3, 3)
    }
}

/// Striped Bears — {3}{G} 2/2 that replaces itself.
pub fn striped_bears() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_draw(1)],
        ..creature("Striped Bears", cost(&[generic(3), g()]), vec![CreatureType::Bear], 2, 2)
    }
}

/// Merfolk Traders — {1}{U} 1/2 that loots on arrival.
pub fn merfolk_traders() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_loot()],
        ..creature("Merfolk Traders", cost(&[generic(1), u()]), vec![CreatureType::Merfolk], 1, 2)
    }
}

/// Lava Hounds — {2}{R}{R} 4/4 haste that bites its own controller.
pub fn lava_hounds() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(deal(4, Selector::You))],
        ..creature("Lava Hounds", cost(&[generic(2), r(), r()]), vec![CreatureType::Dog], 4, 4)
    }
}

/// Tolarian Serpent — {5}{U}{U} 7/7 that mills you seven each upkeep.
pub fn tolarian_serpent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::Mill { who: Selector::You, amount: Value::Const(7) },
        }],
        ..creature(
            "Tolarian Serpent",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Serpent],
            7,
            7,
        )
    }
}

/// Odylic Wraith — {3}{B} 2/2 swampwalker whose hits strip a card.
pub fn odylic_wraith() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature("Odylic Wraith", cost(&[generic(3), b()]), vec![CreatureType::Wraith], 2, 2)
    }
}

/// Timid Drake — {2}{U} 3/3 flier that bounces itself when anything else lands.
pub fn timid_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                },
            ),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
        }],
        ..creature("Timid Drake", cost(&[generic(2), u()]), vec![CreatureType::Drake], 3, 3)
    }
}

/// Mischievous Poltergeist — {2}{B} 1/1 flier that regenerates for life.
pub fn mischievous_poltergeist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Mischievous Poltergeist", cost(&[generic(2), b()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Southern Paladin — {2}{W}{W} 3/3 red-hoser.
pub fn southern_paladin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w()]),
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Permanent.and(R::HasColor(Color::Red))),
            },
            ..Default::default()
        }],
        ..creature(
            "Southern Paladin",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Master of Arms — {2}{W} 2/2 first striker that taps its own blockers.
pub fn master_of_arms() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::BlockingOrBlockedBySource)),
            },
            ..Default::default()
        }],
        ..creature(
            "Master of Arms",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Llanowar Behemoth — {3}{G}{G} 4/4 that eats its team's untapped bodies.
pub fn llanowar_behemoth() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Creature.and(R::ControlledByYou).and(R::Untapped), 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Llanowar Behemoth",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Elemental],
            4,
            4,
        )
    }
}

/// Llanowar Druid — {1}{G} 1/2 that cashes itself in for a Forest untap.
pub fn llanowar_druid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Untap {
                what: Selector::EachPermanent(R::HasLandType(LandType::Forest)),
                up_to: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Llanowar Druid",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            1,
            2,
        )
    }
}

/// Rogue Elephant — {G} 3/3 that costs a Forest off the battlefield.
pub fn rogue_elephant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessSacrifice {
            filter: R::HasLandType(LandType::Forest),
        })],
        ..creature("Rogue Elephant", cost(&[g()]), vec![CreatureType::Elephant], 3, 3)
    }
}

/// Harvest Wurm — {1}{G} 3/2 that buys back a basic land instead of dying.
pub fn harvest_wurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessCost {
            cost: WardCost::ReturnMatchingFromGraveyardToHand(Box::new(
                R::IsBasicLand.and(R::InYourGraveyard),
            )),
        })],
        ..creature("Harvest Wurm", cost(&[generic(1), g()]), vec![CreatureType::Wurm], 3, 2)
    }
}

/// Peacekeeper — {2}{W} 1/1 that shuts combat off while you pay its upkeep.
pub fn peacekeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(1), w()]) },
        }],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::CantAttack,
            },
        }],
        ..creature("Peacekeeper", cost(&[generic(2), w()]), vec![CreatureType::Human], 1, 1)
    }
}

/// Sylvan Hierophant — {1}{G} 1/2 that trades its own corpse for a better one.
pub fn sylvan_hierophant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Seq(vec![
            Effect::Exile { what: Selector::This },
            Effect::Move {
                what: target_filtered(
                    R::Creature.and(R::InYourGraveyard).and(R::OtherThanSource),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]))],
        ..creature(
            "Sylvan Hierophant",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Touchstone — {2}. Taps down someone else's artifact each turn.
pub fn touchstone() -> CardDefinition {
    artifact(
        "Touchstone",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Artifact.and(R::ControlledByYou.negate())),
            },
            ..Default::default()
        }],
    )
}

/// Serrated Biskelion — {3} 2/2 that shrinks itself to shrink something else.
pub fn serrated_biskelion() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                },
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Serrated Biskelion", cost(&[generic(3)]), vec![CreatureType::Construct], 2, 2)
    }
}

/// Steel Golem — {3} 3/4 that locks you out of casting creatures.
pub fn steel_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "You can't cast creature spells.",
            effect: StaticEffect::ControllerCantCastCreatureSpells,
        }],
        ..creature("Steel Golem", cost(&[generic(3)]), vec![CreatureType::Golem], 3, 4)
    }
}

/// Straw Golem — {1} 2/3 that falls apart when an opponent casts a creature.
pub fn straw_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::SacrificeSource,
        }],
        ..creature("Straw Golem", cost(&[generic(1)]), vec![CreatureType::Golem], 2, 3)
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Serra's Blessing — {1}{W}. Your team stops tapping to attack.
pub fn serras_blessing() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Vigilance,
            },
        }],
        ..enchantment("Serra's Blessing", cost(&[generic(1), w()]))
    }
}

/// Dense Foliage — {2}{G}. Nothing can target a creature with a spell.
pub fn dense_foliage() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures can't be the targets of spells.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::CantBeTargetedBySpells,
            },
        }],
        ..enchantment("Dense Foliage", cost(&[generic(2), g()]))
    }
}

/// Pendrell Mists — {3}{U}. Every creature on the table rents its slot.
pub fn pendrell_mists() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All creatures have \"At the beginning of your upkeep, sacrifice this creature unless you pay {1}.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature,
                ability: Box::new(TriggeredAbility {
                    event: your_upkeep(),
                    effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(1)]) },
                }),
            },
        }],
        ..enchantment("Pendrell Mists", cost(&[generic(3), u()]))
    }
}

/// Heat Stroke — {2}{R}. Everything that met a blocker dies at end of combat.
pub fn heat_stroke() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::EndCombat), EventScope::AnyPlayer),
            effect: Effect::Destroy {
                what: Selector::EachPermanent(
                    R::Creature.and(R::BlockedThisTurn.or(R::WasBlockedThisTurn)),
                ),
            },
        }],
        ..enchantment("Heat Stroke", cost(&[generic(2), r()]))
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Relearn — {1}{U}{U}. Buy back a spell.
pub fn relearn() -> CardDefinition {
    sorcery(
        "Relearn",
        cost(&[generic(1), u(), u()]),
        Effect::Move {
            what: target_filtered(
                R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::InYourGraveyard),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Vitalize — {G}. Untap your whole team.
pub fn vitalize() -> CardDefinition {
    instant(
        "Vitalize",
        cost(&[g()]),
        Effect::Untap {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            up_to: None,
        },
    )
}

/// Nature's Resurgence — {2}{G}{G}. Everyone cashes in their dead.
pub fn natures_resurgence() -> CardDefinition {
    sorcery(
        "Nature's Resurgence",
        cost(&[generic(2), g(), g()]),
        Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::Creature,
                },
            }),
        },
    )
}

/// Shattered Crypt — {X}{B}{B}. X bodies back, X life down.
pub fn shattered_crypt() -> CardDefinition {
    sorcery(
        "Shattered Crypt",
        cost(&[crate::mana::x(), b(), b()]),
        Effect::Seq(vec![
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature.and(R::InYourGraveyard),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                }),
            },
            Effect::LoseLife { who: Selector::You, amount: Value::XFromCost },
        ]),
    )
}

/// Lava Storm — {3}{R}{R}. Two damage to one side of the combat.
pub fn lava_storm() -> CardDefinition {
    instant(
        "Lava Storm",
        cost(&[generic(3), r(), r()]),
        Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::IsBlocking)),
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Thunderbolt — {1}{R}. Three upstairs, or four at a flier.
pub fn thunderbolt() -> CardDefinition {
    instant(
        "Thunderbolt",
        cost(&[generic(1), r()]),
        Effect::ChooseMode(vec![
            deal(3, target_filtered(R::Player.or(R::Planeswalker))),
            deal(4, target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying)))),
        ]),
    )
}

/// Tendrils of Despair — {B}. A creature buys two cards off an opponent.
pub fn tendrils_of_despair() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..sorcery(
            "Tendrils of Despair",
            cost(&[b()]),
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
            },
        )
    }
}

// ── Wave 2: cumulative upkeep, graveyard costs, animation ───────────────────

/// Aboroth — {4}{G}{G} 9/9 that shrinks itself a little more every upkeep.
pub fn aboroth() -> CardDefinition {
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::PutCounterOnSelf(CounterType::MinusOneMinusOne))],
        ..creature("Aboroth", cost(&[generic(4), g(), g()]), vec![CreatureType::Elemental], 9, 9)
    }
}

/// Mwonvuli Ooze — {G} whose body is 1 plus twice its age counters.
pub fn mwonvuli_ooze() -> CardDefinition {
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::Mana(cost(&[generic(2)])))],
        dynamic_pt: Some(crate::card::DynamicPt::BasePlusCountersOnSelf {
            counter_type: CounterType::Age,
            base_p: 1,
            base_t: 1,
            per_p: 2,
            per_t: 2,
        }),
        ..creature("Mwonvuli Ooze", cost(&[g()]), vec![CreatureType::Ooze], 1, 1)
    }
}

/// Psychic Vortex — {2}{U}{U}. Draws you an extra card per age counter, then
/// bills a land and your hand each end step.
pub fn psychic_vortex() -> CardDefinition {
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::Draw(1))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Land,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
            ]),
        }],
        ..enchantment("Psychic Vortex", cost(&[generic(2), u(), u()]))
    }
}

/// Inner Sanctum — {1}{W}{W}. Your creatures take nothing while you keep
/// paying for it in life.
pub fn inner_sanctum() -> CardDefinition {
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::Life(2))],
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to creatures you control.",
            effect: StaticEffect::PreventAllDamageToYourCreatures,
        }],
        ..enchantment("Inner Sanctum", cost(&[generic(1), w(), w()]))
    }
}

/// Wave of Terror — {2}{B}. Each draw step it kills the whole curve slot its
/// age counters have reached.
pub fn wave_of_terror() -> CardDefinition {
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::YourControl),
            effect: Effect::DestroyNoRegen {
                what: Selector::EachPermanent(
                    R::Creature.and(R::ManaValueEqualsCountersOnSource(CounterType::Age)),
                ),
            },
        }],
        ..enchantment("Wave of Terror", cost(&[generic(2), b()]))
    }
}

/// Ancestral Knowledge — {1}{U}. Sculpts the top ten, and unsculpts when it
/// leaves.
pub fn ancestral_knowledge() -> CardDefinition {
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![
            etb(Effect::LookExileAnyNumberRestBack {
                who: Selector::You,
                count: Value::Const(10),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::ShuffleLibrary { who: PlayerRef::You },
            },
        ],
        ..enchantment("Ancestral Knowledge", cost(&[generic(1), u()]))
    }
}

/// Barrow Ghoul — {1}{B} 4/4 that eats a corpse a turn to stay.
pub fn barrow_ghoul() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::SacrificeSourceUnlessCost {
                cost: WardCost::ExileTopFromGraveyardMatching(Box::new(R::Creature)),
            },
        }],
        ..creature("Barrow Ghoul", cost(&[generic(1), b()]), vec![CreatureType::Zombie], 4, 4)
    }
}

/// Circling Vultures — {B} 3/2 flier on the same corpse diet, pitchable at
/// instant speed.
pub fn circling_vultures() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Noop,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::SacrificeSourceUnlessCost {
                cost: WardCost::ExileTopFromGraveyardMatching(Box::new(R::Creature)),
            },
        }],
        ..creature("Circling Vultures", cost(&[b()]), vec![CreatureType::Bird], 3, 2)
    }
}

/// Necratog — {1}{B}{B} 1/2 Atog that eats your graveyard's creatures.
pub fn necratog() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
            exile_other_top: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Necratog", cost(&[generic(1), b(), b()]), vec![CreatureType::Atog], 1, 2)
    }
}

/// Zombie Scavengers — {2}{B} 3/1 that regenerates off the graveyard's top.
pub fn zombie_scavengers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
            exile_other_top: true,
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Zombie Scavengers", cost(&[generic(2), b()]), vec![CreatureType::Zombie], 3, 1)
    }
}

/// Soul Shepherd — {1}{W} 2/1 that trades corpses for life.
pub fn soul_shepherd() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Soul Shepherd",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            1,
        )
    }
}

/// Roc Hatchling — {R} 0/1 that grows up once its shell counters come off.
pub fn roc_hatchling() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Shell, Value::Const(4))),
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::RemoveCounter {
                what: Selector::This,
                kind: CounterType::Shell,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "As long as this creature has no shell counters on it, it gets +3/+2.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::Not(Box::new(Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::Shell,
                        n: 1,
                    })),
                    inner: Box::new(StaticEffect::PumpPT {
                        applies_to: Selector::This,
                        power: 3,
                        toughness: 2,
                    }),
                },
            },
            StaticAbility {
                description: "As long as this creature has no shell counters on it, it has flying.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::Not(Box::new(Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::Shell,
                        n: 1,
                    })),
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: Selector::This,
                        keyword: Keyword::Flying,
                    }),
                },
            },
        ],
        ..creature("Roc Hatchling", cost(&[r()]), vec![CreatureType::Bird], 0, 1)
    }
}

/// Xanthic Statue — {8} that stands up as an 8/8 trampler.
pub fn xanthic_statue() -> CardDefinition {
    artifact(
        "Xanthic Statue",
        cost(&[generic(8)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(8),
                toughness: Value::Const(8),
                creature_types: vec![CreatureType::Golem],
                keywords: vec![Keyword::Trample],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

/// Chimeric Sphere — {3} that picks a body: a 2/1 flier or a 3/2 ground one.
pub fn chimeric_sphere() -> CardDefinition {
    let body = |p: i32, t: i32, keywords: Vec<Keyword>| Effect::BecomeCreature {
        what: Selector::This,
        power: Value::Const(p),
        toughness: Value::Const(t),
        creature_types: vec![CreatureType::Construct],
        keywords,
        duration: Duration::EndOfTurn,
    };
    artifact(
        "Chimeric Sphere",
        cost(&[generic(3)]),
        vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: body(2, 1, vec![Keyword::Flying]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: body(3, 2, vec![]),
                ..Default::default()
            },
        ],
    )
}

/// Thran Forge — {3}. Turns something into a slightly bigger artifact.
pub fn thran_forge() -> CardDefinition {
    artifact(
        "Thran Forge",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::Artifact.negate())),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::AddCardTypeIndefinitely {
                    what: Selector::Target(0),
                    card_type: CardType::Artifact,
                    until_eot: true,
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Jabari's Banner — {2}. Lends flanking for a turn.
pub fn jabaris_banner() -> CardDefinition {
    artifact(
        "Jabari's Banner",
        cost(&[generic(1)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flanking,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

/// Phyrexian Furnace — {1}. Nibbles a graveyard, or cashes itself in for one
/// card and a bigger bite.
pub fn phyrexian_furnace() -> CardDefinition {
    artifact(
        "Phyrexian Furnace",
        cost(&[generic(1)]),
        vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::ExileBottomOfGraveyard { who: PlayerRef::Target(0) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Exile { what: target_filtered(R::InGraveyard) },
                    draw(1),
                ]),
                ..Default::default()
            },
        ],
    )
}

/// Well of Knowledge — {3}. Anyone may buy a card, but only in their draw step.
pub fn well_of_knowledge() -> CardDefinition {
    artifact(
        "Well of Knowledge",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            any_player: true,
            condition: Some(Predicate::CurrentStepIs(TurnStep::Draw)),
            effect: draw(1),
            ..Default::default()
        }],
    )
}
