//! Modern Horizons 2 sweep, batch 8 — echo enforcement (CR 702.29),
//! granted suspend (CR 702.62e), chosen-number coin flips, acorn anthem,
//! counter movement, snow/colorless land hate. Tests in `tests/mh2g.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect};
use crate::mana::{b, cost, g, generic, r, u, x};

use SelectionRequirement as R;

fn squirrel_token() -> TokenDefinition {
    TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel],
            ..Default::default()
        },
        colors: vec![crate::mana::Color::Green],
        ..Default::default()
    }
}

/// Chitterspitter — {2}{G} artifact. Upkeep: may sacrifice a token for an
/// acorn counter; Squirrels you control get +1/+1 per acorn counter;
/// {G}, {T}: create a 1/1 green Squirrel.
pub fn chitterspitter() -> CardDefinition {
    CardDefinition {
        name: "Chitterspitter",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a token for an acorn counter?".into(),
                filter: R::IsToken,
                count: Value::ONE,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Acorn,
                    amount: Value::ONE,
                }),
                else_: None,
            },
        }],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Squirrels you control get +1/+1 for each acorn counter on this.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Squirrel),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: Some(CounterType::Acorn),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[g()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: squirrel_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chrome Courier — {1}{W}{U} 1/1 flying Thopter. ETB: reveal top two, one
/// to hand, other to graveyard; gain 3 if an artifact went to hand.
pub fn chrome_courier() -> CardDefinition {
    CardDefinition {
        name: "Chrome Courier",
        cost: cost(&[generic(1), crate::mana::w(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(2),
            rest_to_graveyard: true,
            pick_filter: None,
            take: None,
            to_battlefield: false,
            gain_life_if_pick: Some((R::Artifact, 3)),
            gain_life_greatest_power_rest: false,
            optional: false,
        })],
        ..Default::default()
    }
}

/// Discerning Taste — {2}{B} sorcery. Look at top four, one to hand, rest to
/// graveyard; gain life equal to the greatest power milled this way.
pub fn discerning_taste() -> CardDefinition {
    CardDefinition {
        name: "Discerning Taste",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: true,
            pick_filter: None,
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: true,
            optional: false,
        },
        ..Default::default()
    }
}

/// Break the Ice — {B}{B} sorcery. Destroy target land that is snow or could
/// produce {C}. Overload {4}{B}{B}.
pub fn break_the_ice() -> CardDefinition {
    use crate::card::AlternativeCost;
    let filter = R::Land.and(R::IsSnow.or(R::ProducesColorless));
    CardDefinition {
        name: "Break the Ice",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: target_filtered(filter.clone()) },
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(4), b(), b()]),
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(filter),
                body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Obsidian Charmaw — {3}{R}{R} 4/4 flying Dragon. Costs {1} less per
/// opponent land that could produce {C}; ETB: destroy target nonbasic land
/// an opponent controls.
pub fn obsidian_charmaw() -> CardDefinition {
    CardDefinition {
        name: "Obsidian Charmaw",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Costs {1} less per opponent land that could produce {C}.",
            effect: StaticEffect::SelfCostReducedPerPermanentMatching {
                filter: R::Land.and(R::ControlledByOpponent).and(R::ProducesColorless),
                per: 1,
            },
        }],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::IsNonbasicLand.and(R::ControlledByOpponent)),
        })],
        ..Default::default()
    }
}

/// Rakdos Headliner — {B}{R} 3/3 Devil with haste and Echo—Discard a card.
pub fn rakdos_headliner() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Headliner",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste, Keyword::EchoDiscard],
        ..Default::default()
    }
}

/// Wren's Run Hydra — {X}{G} 0/0 reach Hydra with X +1/+1 counters.
/// Reinforce X—{X}{G}{G} (discard-activated: X counters on target creature).
pub fn wrens_run_hydra() -> CardDefinition {
    CardDefinition {
        name: "Wren's Run Hydra",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hydra],
            ..Default::default()
        },
        keywords: vec![Keyword::Reach],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), g(), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ghost-Lit Drifter — {2}{U} 2/2 flying Spirit. {2}{U}: another target
/// creature gains flying EOT. Channel—{X}{U}, Discard: target creature gains
/// flying (the printed "X target creatures" is modeled as one target).
pub fn ghost_lit_drifter() -> CardDefinition {
    let fly = |filter: R| Effect::GrantKeyword {
        what: target_filtered(filter),
        keyword: Keyword::Flying,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Ghost-Lit Drifter",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                effect: fly(R::Creature.and(R::OtherThanSource)),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[x(), u()]),
                from_hand: true,
                discard_self_cost: true,
                effect: fly(R::Creature),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Steel Dromedary — {3} 2/2 Camel. Enters tapped with two +1/+1 counters;
/// doesn't untap while it has a +1/+1 counter; at combat on your turn, may
/// move a +1/+1 counter onto target creature.
pub fn steel_dromedary() -> CardDefinition {
    CardDefinition {
        name: "Steel Dromedary",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Camel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::DoesntUntapWhileCounter(CounterType::PlusOnePlusOne)],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        static_abilities: vec![crate::card::StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::MayDo {
                description: "Move a +1/+1 counter onto target creature?".into(),
                body: Box::new(Effect::MoveCounters {
                    from: Selector::This,
                    to: target_filtered(R::Creature.and(R::OtherThanSource)),
                    counter: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Suspend — {U} instant. Exile target creature with two time counters; if it
/// doesn't have suspend, it gains suspend (CR 702.62e).
pub fn suspend() -> CardDefinition {
    CardDefinition {
        name: "Suspend",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantSuspend {
            what: target_filtered(R::Creature),
            time_counters: 2,
        },
        ..Default::default()
    }
}

/// Yusri, Fortune's Flame — {1}{U}{R} 2/3 flying Efreet. On attack: choose
/// 1–5, flip that many coins; draw per win, take 2 per loss; five wins =
/// free spells from hand this turn.
pub fn yusri_fortunes_flame() -> CardDefinition {
    CardDefinition {
        name: "Yusri, Fortune's Flame",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Efreet],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::FlipCoinsChooseCount {
                max: 5,
                per_win: Box::new(draw(1)),
                per_loss: Box::new(Effect::DealDamage {
                    amount: Value::Const(2),
                    to: Selector::You,
                }),
                all_won: Box::new(Effect::FreeSpellsFromHandThisTurn),
                all_won_min: 5,
            },
        }],
        ..Default::default()
    }
}

/// Aeve, Progenitor Ooze — {2}{G}{G}{G} 2/2 storm Ooze; isn't legendary as a
/// token; enters with a +1/+1 counter per other Ooze you control.
pub fn aeve_progenitor_ooze() -> CardDefinition {
    CardDefinition {
        name: "Aeve, Progenitor Ooze",
        cost: cost(&[generic(2), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Storm],
        nonlegendary_as_token: true,
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::count(Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Ooze)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
            )),
        )),
        ..Default::default()
    }
}
