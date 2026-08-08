//! Nemesis (NMS), fourth wave — the set's last sixteen cards. Tests in
//! `classic_sets/nms4`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{LookPick, 
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest, shortcut::target_filtered,
};
use crate::game::TurnStep;
use crate::mana::{ManaCost, cost, g, generic, r, u, w};

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

/// "The number of fade counters on Saproling Burst" — read off the token's
/// creator, so a copy of the token scales with whatever minted it.
fn fade_counters_on_creator() -> Value {
    Value::CountersOn { what: Box::new(Selector::CreatorOfSource), kind: CounterType::Fade }
}

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// Defiant Vanguard — {2}{W} 2/2. Blocks once, then takes everything it
/// blocked with it at end of combat.
pub fn defiant_vanguard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Seq(vec![
                    Effect::Destroy { what: Selector::CreaturesBlockedBySourceThisTurn },
                    Effect::Destroy { what: Selector::This },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::PermanentCard
                    .and(R::HasCreatureType(CreatureType::Rebel))
                    .and(R::ManaValueAtMost(4)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Defiant Vanguard", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Rebel], 2, 2)
    }
}

/// Divining Witch — {1}{B} 1/1 Spellshaper. Names a card, burns six off the
/// top, and digs the named card out of the rest.
pub fn divining_witch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), crate::mana::b()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::NameCard { what: Selector::This, restrict_to: None },
                Effect::ExileTopThenRevealUntilNamed { exile_count: Value::Const(6) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Divining Witch",
            cost(&[generic(1), crate::mana::b()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Eye of Yawgmoth — {3}. Trades a creature for a dig as deep as its power.
pub fn eye_of_yawgmoth() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::SacrificedPower,
                rest_to_exile: true,
    ..Default::default()
})),
            ..Default::default()
        }],
        ..artifact("Eye of Yawgmoth", cost(&[generic(3)]))
    }
}

/// Flowstone Armor — {3}. Stays tapped to hold a −1 toughness on something.
pub fn flowstone_armor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::WhileSourceTapped,
            },
            ..Default::default()
        }],
        ..artifact("Flowstone Armor", cost(&[generic(3)]))
    }
}

/// Fog Patch — {1}{G}. Blanks the whole attack after blockers are in.
pub fn fog_patch() -> CardDefinition {
    CardDefinition {
        name: "Fog Patch",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::AttackingCreaturesBecomeBlocked,
        cast_only_after_blockers: true,
        ..Default::default()
    }
}

/// Harvest Mage — {G} 1/1 Spellshaper. Turns your lands into a one-colour
/// fixer for the turn.
pub fn harvest_mage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[g()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::ReplaceLandManaThisTurn {
                mine_only: true,
                nonbasic_only: false,
                color_of_choice: true,
            },
            ..Default::default()
        }],
        ..creature(
            "Harvest Mage",
            cost(&[g()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Kill Switch — {3}. Taps every other artifact down and holds them there
/// while it stays tapped.
pub fn kill_switch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::TapAndUntapLock {
                what: Selector::EachPermanent(R::Artifact.and(R::OtherThanSource)),
            },
            ..Default::default()
        }],
        ..artifact("Kill Switch", cost(&[generic(3)]))
    }
}

/// Mana Cache — {1}{R}{R}. Banks a counter per untapped land each end step;
/// anyone may cash them in on their own turn.
pub fn mana_cache() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::PermanentCountControlledByMatching(
                    PlayerRef::ActivePlayer,
                    R::Land.and(R::Untapped),
                ),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Charge, 1)),
            any_player: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Not(Box::new(Predicate::CurrentStepIs(TurnStep::End))),
            ])),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::ONE),
            },
            ..Default::default()
        }],
        ..enchantment("Mana Cache", cost(&[generic(1), r(), r()]))
    }
}

/// Mogg Toady — {1}{R} 2/2. Only fights when it has the numbers.
pub fn mogg_toady() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantAttackUnlessMoreCreaturesThanDefender,
            Keyword::CantBlockUnlessMoreCreaturesThanAttacker,
        ],
        ..creature("Mogg Toady", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 2, 2)
    }
}

/// Oracle's Attendants — {3}{W} 1/5. Soaks up everything one source would
/// throw at a creature this turn.
pub fn oracles_attendants() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                to: Some(target_filtered(R::Creature)),
                redirect_to: Some(Selector::This),
                whole_turn: true,
                exile_top_per_prevented: false,
                reflect: false,
                gain_life: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Oracle's Attendants",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            5,
        )
    }
}

/// Overlaid Terrain — {2}{G}{G}. Burns your board of lands for double mana
/// out of what's left.
pub fn overlaid_terrain() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::SacrificeAllMatching {
            who: Selector::Player(PlayerRef::You),
            filter: R::Land,
        }),
        static_abilities: vec![StaticAbility {
            description: "Lands you control have \"{T}: Add two mana of any one color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyOneColor(Value::Const(2)),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..enchantment("Overlaid Terrain", cost(&[generic(2), g(), g()]))
    }
}

/// Pale Moon — {1}{U}. Every nonbasic land is a Wastes for the turn.
pub fn pale_moon() -> CardDefinition {
    CardDefinition {
        name: "Pale Moon",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ReplaceLandManaThisTurn {
            mine_only: false,
            nonbasic_only: true,
            color_of_choice: false,
        },
        ..Default::default()
    }
}

/// Rootwater Thief — {1}{U} 1/2. Flies over and strips a card of your choice
/// out of their deck.
pub fn rootwater_thief() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2} to exile a card from that player's library?".to_string(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::SearchPickedBy {
                    who: PlayerRef::TriggerEventPlayer,
                    picker: PlayerRef::You,
                    filter: R::Any,
                    to: ZoneDest::Exile,
                }),
                else_: None,
            },
        }],
        ..creature(
            "Rootwater Thief",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Rogue],
            1,
            2,
        )
    }
}

/// Saproling Burst — {4}{G}. Fading 7, cashed one counter at a time for a
/// Saproling as big as what's left; they all die with it.
pub fn saproling_burst() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(7)],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Fade, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Saproling".to_string(),
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Saproling],
                        ..Default::default()
                    },
                    colors: vec![crate::mana::Color::Green],
                    static_abilities: vec![StaticAbility {
                        description: "This token's power and toughness are each equal to the number of fade counters on Saproling Burst.",
                        effect: StaticEffect::SelfBasePtFromValue {
                            power: fade_counters_on_creator(),
                            toughness: fade_counters_on_creator(),
                        },
                    }],
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CantBeRegeneratedThisTurn { what: Selector::TokensCreatedBySource },
                Effect::Destroy { what: Selector::TokensCreatedBySource },
            ]),
        }],
        ..enchantment("Saproling Burst", cost(&[generic(4), g()]))
    }
}

/// Sivvi's Valor — {2}{W}. Takes a creature's damage on the chin; free if you
/// have a Plains and a creature to tap.
pub fn sivvis_valor() -> CardDefinition {
    CardDefinition {
        name: "Sivvi's Valor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventAllDamageThisTurn {
            target: target_filtered(R::Creature),
            redirect_to: Some(Selector::Player(PlayerRef::You)),
        },
        alternative_cost: Some(AlternativeCost {
            tap_creatures: Some((R::Creature, 1)),
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::HasLandType(LandType::Plains).and(R::ControlledByYou),
                ),
                n: Value::ONE,
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Stronghold Gambit — {1}{R}. Everyone shows a card; the cheapest creature
/// revealed hits play free.
pub fn stronghold_gambit() -> CardDefinition {
    CardDefinition {
        name: "Stronghold Gambit",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealChosenCardsLowestCreaturesEnter,
        ..Default::default()
    }
}
