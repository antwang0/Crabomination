//! Weatherlight (WTH) closing waves. Tests in `classic_sets/wth`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, StaticAbility, StateTriggeredAbility, Subtypes, TriggeredAbility,
    WardCost, Zone,
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

// ── Wave 3: Auras, combat tricks, graveyard value ───────────────────────────

fn aura(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// Abduction — {2}{U}{U} Aura. You keep the creature, and its owner gets it
/// back on the battlefield when it dies.
pub fn abduction() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Untap { what: enchanted(), up_to: None },
                Effect::GainControlWhileSourceRemains { what: enchanted() },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
                effect: Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                        tapped: false,
                    },
                },
            },
        ],
        ..aura("Abduction", cost(&[generic(2), u(), u()]), EquipBonus::default())
    }
}

/// Apathy — {U} Aura. The creature stays tapped unless its controller pitches
/// a card at random.
pub fn apathy() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: enchanted() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::ControllerOf(Box::new(enchanted())))),
            effect: Effect::MayDoBy {
                who: PlayerRef::ControllerOf(Box::new(enchanted())),
                description: "Discard a card at random to untap the enchanted creature?"
                    .to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
                    Effect::Untap { what: enchanted(), up_to: None },
                ])),
            },
        }],
        ..aura("Apathy", cost(&[u()]), EquipBonus::default())
    }
}

/// Mana Chains — {U} Aura that saddles the creature with cumulative upkeep {1}.
pub fn mana_chains() -> CardDefinition {
    aura(
        "Mana Chains",
        cost(&[u()]),
        EquipBonus {
            keywords: vec![cu(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
            ..Default::default()
        },
    )
}

/// Kithkin Armor — {W} Aura. Big blockers bounce off, and it can be cashed in
/// as a damage shield.
pub fn kithkin_armor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: Some(enchanted()),
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..aura(
            "Kithkin Armor",
            cost(&[w()]),
            EquipBonus {
                keywords: vec![Keyword::CantBeBlockedByPowerAtLeast(3)],
                ..Default::default()
            },
        )
    }
}

/// Betrothed of Fire — {1}{R} Aura. Feed it creatures for a pump, or cash the
/// host in for a team pump.
pub fn betrothed_of_fire() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                sac_other_filter: Some((R::Creature.and(R::ControlledByYou).and(R::Untapped), 1)),
                effect: Effect::PumpPT {
                    what: enchanted(),
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_other_filter: Some((R::AttachedToSource, 1)),
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..aura("Betrothed of Fire", cost(&[generic(1), r()]), EquipBonus::default())
    }
}

/// Nature's Kiss — {1}{G} Aura. Buys pump with graveyard cards.
pub fn natures_kiss() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            exile_other_filter: Some((R::InYourGraveyard, 1)),
            exile_other_top: true,
            effect: Effect::PumpPT {
                what: enchanted(),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..aura("Nature's Kiss", cost(&[generic(1), g()]), EquipBonus::default())
    }
}

/// Alms — {W}. Buys damage prevention with graveyard cards.
pub fn alms() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            exile_other_filter: Some((R::InYourGraveyard, 1)),
            exile_other_top: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Alms", cost(&[w()]))
    }
}

/// Teferi's Veil — {1}{U}. Your attackers blink out of range after combat.
pub fn teferis_veil() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::PhaseOut {
                    what: Selector::TriggerSource,
                    until_source_leaves: false,
                }),
            },
        }],
        ..enchantment("Teferi's Veil", cost(&[generic(1), u()]))
    }
}

// ── Wave 3 creatures ────────────────────────────────────────────────────────

/// Bone Dancer — {1}{B}{B} 2/2 that raids the defender's graveyard when it
/// gets through.
pub fn bone_dancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_unblocked(Effect::MayDo {
            description: "Reanimate the top creature card of defending player's graveyard?"
                .to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::DefendingPlayer,
                        zone: Zone::Graveyard,
                        filter: R::Creature,
                    }),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::DealsNoCombatDamage,
                    duration: Duration::EndOfTurn,
                },
            ])),
        })],
        ..creature("Bone Dancer", cost(&[generic(1), b(), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Goblin Vandal — {R} 1/1 that trades its damage for an artifact.
pub fn goblin_vandal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_unblocked(Effect::MayPay {
            description: "Pay {R} to destroy an artifact the defending player controls?".to_string(),
            mana_cost: cost(&[r()]),
            else_: None,
            body: Box::new(Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(
                        R::Artifact.and(R::ControlledByYou.negate()),
                    ),
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::DealsNoCombatDamage,
                    duration: Duration::EndOfTurn,
                },
            ])),
        })],
        ..creature(
            "Goblin Vandal",
            cost(&[r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            1,
            1,
        )
    }
}

/// Goblin Grenadiers — {3}{R} 2/2 that cashes itself in for a creature and a
/// land when it connects.
pub fn goblin_grenadiers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_unblocked(Effect::MayDo {
            description: "Sacrifice Goblin Grenadiers to destroy a creature and a land?"
                .to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::Destroy { what: target_filtered(R::Creature) },
                Effect::Destroy { what: Selector::TargetFiltered { slot: 1, filter: R::Land } },
            ])),
        })],
        ..creature("Goblin Grenadiers", cost(&[generic(3), r()]), vec![CreatureType::Goblin], 2, 2)
    }
}

/// Sawtooth Ogre — {2}{R}{R} 3/3 that gets one last lick in after combat.
pub fn sawtooth_ogre() -> CardDefinition {
    let strike_back = || Effect::AtEndOfCombat {
        body: Box::new(deal(1, Selector::CreaturesInCombatWith(Box::new(Selector::This)))),
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: strike_back(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: strike_back(),
            },
        ],
        ..creature("Sawtooth Ogre", cost(&[generic(2), r(), r()]), vec![CreatureType::Ogre], 3, 3)
    }
}

/// Tolarian Entrancer — {1}{U} 1/1 that keeps whatever blocks it.
pub fn tolarian_entrancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::GainControl {
                    what: Selector::TriggerSource,
                    to: None,
                    duration: Duration::Permanent,
                }),
            },
        }],
        ..creature(
            "Tolarian Entrancer",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Veteran Explorer — {G} 1/1 whose death ramps the whole table.
pub fn veteran_explorer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                count: Value::Const(2),
            }),
        })],
        ..creature(
            "Veteran Explorer",
            cost(&[g()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Noble Benefactor — {2}{U} 2/2 whose death tutors for everyone.
pub fn noble_benefactor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::MayDo {
                description: "Search your library for a card?".to_string(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Any,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            }),
        })],
        ..creature(
            "Noble Benefactor",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Urborg Stalker — {3}{B} 2/4 that punishes anyone playing off-colour.
pub fn urborg_stalker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::SelectorExists(Selector::EachPermanent(
                    R::Nonland
                        .and(R::HasColor(Color::Black).negate())
                        .and(R::ControlledByActivePlayer),
                ))),
            effect: deal(1, Selector::Player(PlayerRef::ActivePlayer)),
        }],
        ..creature("Urborg Stalker", cost(&[generic(3), b()]), vec![CreatureType::Horror], 2, 4)
    }
}

/// Manta Ray — {1}{U}{U} 3/3 that only works in an Islands matchup.
pub fn manta_ray() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantAttackUnlessDefenderControlsLandType(LandType::Island),
            Keyword::CantBeBlockedExceptBy(Box::new(R::HasColor(Color::Blue))),
        ],
        state_trigger: Some(StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::SelectorExists(
                Selector::EachPermanent(R::HasLandType(LandType::Island).and(R::ControlledByYou)),
            ))),
            effect: Effect::SacrificeSource,
        }),
        ..creature("Manta Ray", cost(&[generic(1), u(), u()]), vec![CreatureType::Fish], 3, 3)
    }
}

/// Llanowar Sentinel — {2}{G} 2/3 that calls in its twin.
pub fn llanowar_sentinel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {1}{G} to search for another Llanowar Sentinel?".to_string(),
            mana_cost: cost(&[generic(1), g()]),
            else_: None,
            body: Box::new(Effect::SearchSameNameToBattlefield {
                who: PlayerRef::You,
                what: Selector::This,
            }),
        })],
        ..creature("Llanowar Sentinel", cost(&[generic(2), g()]), vec![CreatureType::Elf], 2, 3)
    }
}

/// Fungus Elemental — {3}{G} 3/3 that can eat Forests the turn it lands.
pub fn fungus_elemental() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_other_filter: Some((R::HasLandType(LandType::Forest), 1)),
            condition: Some(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::EnteredThisTurn,
            }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusTwoPlusTwo,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Fungus Elemental",
            cost(&[generic(3), g()]),
            vec![CreatureType::Fungus, CreatureType::Elemental],
            3,
            3,
        )
    }
}

/// Benalish Missionary — {W} 1/1 that blanks a blocked attacker's damage.
pub fn benalish_missionary() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            effect: Effect::PreventCombatDamageByTargetThisTurn {
                target: target_filtered(R::Creature.and(R::IsBlocked)),
            },
            ..Default::default()
        }],
        ..creature(
            "Benalish Missionary",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

// ── Wave 3 spells ───────────────────────────────────────────────────────────

/// Debt of Loyalty — {1}{W}{W}. Regenerate it, then keep it.
pub fn debt_of_loyalty() -> CardDefinition {
    instant(
        "Debt of Loyalty",
        cost(&[generic(1), w(), w()]),
        Effect::RegenerateThenGainControl { what: target_filtered(R::Creature) },
    )
}

/// Gaea's Blessing — {1}{G}. Shuffles cards back and reshuffles the graveyard
/// if it's milled.
pub fn gaeas_blessing() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardMilled, EventScope::SelfSource),
            effect: Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::You },
        }],
        ..sorcery(
            "Gaea's Blessing",
            cost(&[generic(1), g()]),
            Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 3,
                    min_targets: 0,
                    filter: R::InGraveyard,
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Library {
                            who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                            pos: crate::effect::LibraryPosition::Shuffled,
                        },
                    }),
                },
                draw(1),
            ]),
        )
    }
}

/// Paradigm Shift — {1}{U}. Trade your library for your graveyard.
pub fn paradigm_shift() -> CardDefinition {
    sorcery(
        "Paradigm Shift",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::ExileLibraryExceptBottom { who: PlayerRef::You, keep: Value::ZERO },
            Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::You },
        ]),
    )
}

/// Urborg Justice — {B}{B}. An edict for each of your own dead.
pub fn urborg_justice() -> CardDefinition {
    instant(
        "Urborg Justice",
        cost(&[b(), b()]),
        Effect::Sacrifice {
            who: Selector::Player(PlayerRef::Target(0)),
            count: Value::CreaturesDiedThisTurn(PlayerRef::You),
            filter: R::Creature,
        },
    )
}

/// Liege of the Hollows — {2}{G}{G} 3/4 whose death mints Squirrels for
/// however much mana each player wants to sink.
pub fn liege_of_the_hollows() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::MayPayGenericUpTo {
                max: Value::Const(10),
                body: Box::new(crate::effect::shortcut::mint_token(
                    crate::card::TokenDefinition {
                        name: "Squirrel".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Squirrel],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    1,
                )),
            }),
        })],
        ..creature(
            "Liege of the Hollows",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Spirit],
            3,
            4,
        )
    }
}

// ── Wave 4: the closers ─────────────────────────────────────────────────────

fn land(name: &'static str, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: abilities,
        ..Default::default()
    }
}

/// Abeyance — {1}{W}. Shuts a player's instants, sorceries and non-mana
/// abilities off for the turn, and replaces itself.
pub fn abeyance() -> CardDefinition {
    instant(
        "Abeyance",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::PlayerCantCastMatchingThisTurn {
                who: PlayerRef::Target(0),
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            },
            Effect::PlayerCantActivateNonManaAbilitiesThisTurn { who: PlayerRef::Target(0) },
            draw(1),
        ]),
    )
}

/// Agonizing Memories — {2}{B}{B}. Two of their best cards go back on top.
pub fn agonizing_memories() -> CardDefinition {
    sorcery(
        "Agonizing Memories",
        cost(&[generic(2), b(), b()]),
        Effect::Seq(vec![
            Effect::LookAtHand { who: Selector::Player(PlayerRef::Target(0)) },
            Effect::ChooseFromHandToTopOfLibrary {
                who: PlayerRef::Target(0),
                count: Value::Const(2),
            },
        ]),
    )
}

/// Avizoa — {3}{U} 2/2 flier that trades untap steps for size.
pub fn avizoa() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            once_per_turn: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::SkipPlayerUntapStep { player: PlayerRef::You },
            ]),
            ..Default::default()
        }],
        ..creature("Avizoa", cost(&[generic(3), u()]), vec![CreatureType::Jellyfish], 2, 2)
    }
}

/// Bösium Strip — {3}. Rents the top of your graveyard for a turn.
pub fn bosium_strip() -> CardDefinition {
    artifact(
        "Bösium Strip",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::CastFromGraveyardTopThisTurn,
            ..Default::default()
        }],
    )
}

/// Call of the Wild — {2}{G}{G}. Flips your library for creatures.
pub fn call_of_the_wild() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            effect: Effect::RevealTopDeployIfMatch {
                filter: R::Creature,
                haste: false,
                sacrifice_at_next_end_step: false,
                miss_to_graveyard: true,
            },
            ..Default::default()
        }],
        ..enchantment("Call of the Wild", cost(&[generic(2), g(), g()]))
    }
}

/// Choking Vines — {X}{G}. Blanks X attackers mid-combat and nicks each.
pub fn choking_vines() -> CardDefinition {
    CardDefinition {
        cast_condition: Some(Predicate::CurrentStepIs(TurnStep::DeclareBlockers)),
        ..instant(
            "Choking Vines",
            cost(&[crate::mana::x(), g()]),
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature.and(R::IsAttacking),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::BecomeBlocked { what: Selector::Target(0) },
                        deal(1, Selector::Target(0)),
                    ])),
                }),
            },
        )
    }
}

/// Desperate Gambit — {R}. One flip: double the next hit, or waste it.
pub fn desperate_gambit() -> CardDefinition {
    instant(
        "Desperate Gambit",
        cost(&[r()]),
        Effect::CoinFlipDoubleOrPreventNextDamage { filter: R::Permanent },
    )
}

/// Doomsday — {B}{B}{B}. Five cards left, half your life gone.
pub fn doomsday() -> CardDefinition {
    sorcery("Doomsday", cost(&[b(), b(), b()]), Effect::Doomsday)
}

/// Ertai's Familiar — {1}{U} 2/2 that mills you every time it blinks out.
pub fn ertais_familiar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Phasing],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PhasesOut, EventScope::SelfSource),
                effect: Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            },
        ],
        // Losing Phasing until the next untap step is exactly "can't phase out
        // until your next upkeep" — phasing is checked at untap.
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::LoseKeyword {
                what: Selector::This,
                keyword: Keyword::Phasing,
                duration: Duration::UntilYourNextUntap,
            },
            ..Default::default()
        }],
        ..creature("Ertai's Familiar", cost(&[generic(1), u()]), vec![CreatureType::Illusion], 2, 2)
    }
}

/// Firestorm — {R}. Pitch your hand, spray it across the board.
pub fn firestorm() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::DiscardXFromCost],
        ..instant(
            "Firestorm",
            cost(&[r()]),
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Any,
                    effect: Box::new(Effect::DealDamage {
                        to: Selector::Target(0),
                        amount: Value::XFromCost,
                    }),
                }),
            },
        )
    }
}

/// Haunting Misery — {1}{B}{B}. Your dead pay the damage.
pub fn haunting_misery() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![
            crate::card::AdditionalCastCost::ExileFromGraveyardXFromCost {
                filter: R::Creature.and(R::InYourGraveyard),
            },
        ],
        ..sorcery(
            "Haunting Misery",
            cost(&[generic(1), b(), b()]),
            Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::XFromCost,
            },
        )
    }
}

/// Heart of Bogardan — {2}{R}{R}. When its cumulative upkeep goes unpaid, it
/// takes a player's board with it.
pub fn heart_of_bogardan() -> CardDefinition {
    // X = twice the age counters on it, minus 2. The trigger's event amount
    // is the age count read while it was still on the battlefield.
    let x = || {
        Value::NonNeg(Box::new(Value::Diff(
            Box::new(Value::Times(
                Box::new(Value::TriggerEventAmount),
                Box::new(Value::Const(2)),
            )),
            Box::new(Value::Const(2)),
        )))
    };
    CardDefinition {
        keywords: vec![cu(CumulativeUpkeepCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CumulativeUpkeepUnpaid, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: x(),
                },
                Effect::DealDamage {
                    to: Selector::ControlledBy {
                        who: PlayerRef::Target(0),
                        filter: R::Creature,
                    },
                    amount: x(),
                },
            ]),
        }],
        ..enchantment("Heart of Bogardan", cost(&[generic(2), r(), r()]))
    }
}

/// Lotus Vale — a land that costs two untapped lands and pays three mana back.
pub fn lotus_vale() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessCost {
            cost: WardCost::SacrificeMatchingN(
                Box::new(R::Land.and(R::Untapped).and(R::OtherThanSource)),
                2,
            ),
        })],
        ..land(
            "Lotus Vale",
            vec![ActivatedAbility {
                tap_cost: true,
                effect: crate::effect::shortcut::add_any_one_color(3),
                ..Default::default()
            }],
        )
    }
}

/// Scorched Ruins — Lotus Vale's colorless twin: two lands in, {C}{C}{C}{C} out.
pub fn scorched_ruins() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessCost {
            cost: WardCost::SacrificeMatchingN(
                Box::new(R::Land.and(R::Untapped).and(R::OtherThanSource)),
                2,
            ),
        })],
        ..land(
            "Scorched Ruins",
            vec![ActivatedAbility {
                tap_cost: true,
                effect: crate::effect::shortcut::add_colorless(4),
                ..Default::default()
            }],
        )
    }
}

/// Mana Web — {3}. Tapping one land for mana locks down every land like it.
pub fn mana_web() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Land,
                }),
            effect: Effect::TapLandsSharingProductionWith { land: Selector::TriggerSource },
        }],
        ..artifact("Mana Web", cost(&[generic(3)]), vec![])
    }
}

/// Orcish Settlers — {1}{R} 1/1 that cashes itself in for X lands.
pub fn orcish_settlers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x(), crate::mana::x(), r()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Land,
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
            ..Default::default()
        }],
        ..creature("Orcish Settlers", cost(&[generic(1), r()]), vec![CreatureType::Orc], 1, 1)
    }
}

/// Spinning Darkness — {4}{B}{B}. Free if your graveyard is black enough.
pub fn spinning_darkness() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            exile_filter: Some(R::HasColor(Color::Black).and(R::InYourGraveyard)),
            exile_from_graveyard_count: 3,
            ..Default::default()
        }),
        ..instant(
            "Spinning Darkness",
            cost(&[generic(4), b(), b()]),
            Effect::Seq(vec![
                deal(3, target_filtered(R::Creature.and(R::HasColor(Color::Black).negate()))),
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ]),
        )
    }
}

/// Strands of Night — {2}{B}{B}. Swamps buy your creatures back.
pub fn strands_of_night() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            life_cost: 2,
            sac_other_filter: Some((R::HasLandType(LandType::Swamp), 1)),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..enchantment("Strands of Night", cost(&[generic(2), b(), b()]))
    }
}

/// Tariff — {1}{W}. Everyone's biggest creature pays rent or dies.
pub fn tariff() -> CardDefinition {
    sorcery(
        "Tariff",
        cost(&[generic(1), w()]),
        Effect::EachPlayerSacrificesGreatestManaValueUnlessPays,
    )
}

/// Thran Tome — {4}. Two cards, minus whichever one an opponent bins.
pub fn thran_tome() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Book],
            ..Default::default()
        },
        ..artifact(
            "Thran Tome",
            cost(&[generic(4)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::RevealTopOpponentBinsOne { count: 3, rest_stay_on_top: true },
                    draw(2),
                ]),
                ..Default::default()
            }],
        )
    }
}

/// Winding Canyons — a land that hands your creatures flash for the turn.
pub fn winding_canyons() -> CardDefinition {
    land(
        "Winding Canyons",
        vec![
            ActivatedAbility {
                tap_cost: true,
                effect: crate::effect::shortcut::add_colorless(1),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::GrantCreatureSpellsFlashThisTurn { who: PlayerRef::You },
                ..Default::default()
            },
        ],
    )
}
