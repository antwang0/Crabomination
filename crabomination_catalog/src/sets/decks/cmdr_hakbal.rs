//! Commander: the cards the **Explorers of the Deep** precon (LCC, Hakbal of
//! the Surging Soul) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_hakbal.rs`.
//!
//! Residuals (each also on its card):
//! - **Xolatoyac, the Smiling Flood** — its flooded lands are Islands only
//!   while Xolatoyac is on the battlefield (the ruling keeps them Islands).
//! - **Bygone Marvels** — a copy's new target is the decider's pick, and the
//!   auto-decider keeps the original's (so that copy returns nothing).

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EventKind, EventScope,
    EventSpec, Keyword, LandType, LevelBand, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{encore, etb, mentor, on_attack, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, u, Color, ManaCost};

fn merfolk(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn is_merfolk() -> R {
    R::HasCreatureType(CreatureType::Merfolk)
}

fn your_merfolk() -> Selector {
    Selector::EachPermanent(R::Creature.and(is_merfolk()).and(R::ControlledByYou))
}

fn merfolk_token(hexproof: bool) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Merfolk".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        keywords: if hexproof { vec![Keyword::Hexproof] } else { vec![] },
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk], ..Default::default() },
        ..Default::default()
    })
}

/// "Whenever you cast a Merfolk spell".
fn on_merfolk_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(is_merfolk())),
        effect,
    }
}

/// Hakbal of the Surging Soul — at the beginning of combat on your turn, each
/// Merfolk creature you control explores; attacking, it may put a land from
/// your hand onto the battlefield, else you draw.
pub fn hakbal_of_the_surging_soul() -> CardDefinition {
    let draw = || Effect::Draw { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::Explore { who: your_merfolk() },
            },
            on_attack(Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CardsInHandMatching { who: PlayerRef::You, filter: R::Land },
                    Value::ONE,
                ),
                then: Box::new(Effect::MayDoElse {
                    description: "Put a land card from your hand onto the battlefield? If you don't, draw a card.".into(),
                    body: Box::new(Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::Land,
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    }),
                    else_: Box::new(draw()),
                }),
                else_: Box::new(draw()),
            }),
        ],
        ..legend(merfolk(
            "Hakbal of the Surging Soul",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Scout],
            3,
            3,
        ))
    }
}

/// Bygone Marvels — return target permanent card from your graveyard to hand;
/// Descend 8: the cast copies it twice, new targets allowed. Exiles itself.
/// ⚠ The auto-decider keeps a copy's original target.
pub fn bygone_marvels() -> CardDefinition {
    CardDefinition {
        name: "Bygone Marvels",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Sorcery],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource)
                .with_filter(Predicate::DescendActive { who: PlayerRef::You, count: 8 }),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::Const(2) },
        }],
        effect: Effect::Move {
            what: target_filtered(R::Permanent.from_your_graveyard()),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        exile_on_resolve: true,
        ..Default::default()
    }
}

/// Coralhelm Commander — level up {1}; level 2-3: 3/3 flier; level 4+: 4/4
/// flier and other Merfolk creatures you control get +1/+1.
pub fn coralhelm_commander() -> CardDefinition {
    CardDefinition {
        level_bands: vec![
            LevelBand { min: 2, max: Some(3), power: 3, toughness: 3, keywords: vec![Keyword::Flying] },
            LevelBand { min: 4, max: None, power: 4, toughness: 4, keywords: vec![Keyword::Flying] },
        ],
        static_abilities: vec![StaticAbility {
            description: "LEVEL 4+: Other Merfolk creatures you control get +1/+1.",
            effect: StaticEffect::WhileCountersAtLeast {
                kind: CounterType::Level,
                n: 4,
                inner: Box::new(StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(is_merfolk()).and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Level, amount: Value::ONE },
            ..Default::default()
        }],
        ..merfolk("Coralhelm Commander", cost(&[u(), u()]), vec![CreatureType::Merfolk, CreatureType::Soldier], 2, 2)
    }
}

/// Deeproot Elite — another Merfolk of yours entering puts a +1/+1 counter on
/// target Merfolk you control.
pub fn deeproot_elite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: is_merfolk() },
            ),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(is_merfolk()).and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..merfolk("Deeproot Elite", cost(&[generic(1), g()]), vec![CreatureType::Merfolk, CreatureType::Warrior], 1, 1)
    }
}

/// Deeproot Historian — Merfolk and Druid cards in your graveyard have retrace.
pub fn deeproot_historian() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Merfolk and Druid cards in your graveyard have retrace.",
            effect: StaticEffect::GraveyardCardsHaveRetrace {
                filter: is_merfolk().or(R::HasCreatureType(CreatureType::Druid)),
            },
        }],
        ..merfolk("Deeproot Historian", cost(&[generic(3), g()]), vec![CreatureType::Merfolk, CreatureType::Druid], 3, 3)
    }
}

/// Deeproot Waters — each Merfolk spell makes a 1/1 hexproof Merfolk.
pub fn deeproot_waters() -> CardDefinition {
    CardDefinition {
        name: "Deeproot Waters",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![on_merfolk_cast(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: merfolk_token(true),
        })],
        ..Default::default()
    }
}

/// Emperor Mihail II — look at and cast Merfolk from the top of your library;
/// each Merfolk spell may pay {1} for a 1/1 Merfolk.
pub fn emperor_mihail_ii() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may cast Merfolk spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: is_merfolk().and(R::Not(Box::new(R::Land))) },
            },
        ],
        triggered_abilities: vec![on_merfolk_cast(Effect::MayPay {
            description: "Pay {1} to create a 1/1 Merfolk?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: merfolk_token(false) }),
            else_: None,
        })],
        ..legend(merfolk(
            "Emperor Mihail II",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Noble],
            3,
            3,
        ))
    }
}

/// Herald of Secret Streams — your creatures with +1/+1 counters can't be
/// blocked.
pub fn herald_of_secret_streams() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with +1/+1 counters on them can't be blocked.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                keyword: Keyword::Unblockable,
            },
        }],
        ..merfolk(
            "Herald of Secret Streams",
            cost(&[generic(3), u()]),
            vec![CreatureType::Merfolk, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Kopala, Warden of Waves — opponents' spells and abilities targeting your
/// Merfolk cost {2} more.
pub fn kopala_warden_of_waves() -> CardDefinition {
    let yours = || R::Creature.and(is_merfolk()).and(R::ControlledByYou);
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Spells your opponents cast that target a Merfolk you control cost {2} more to cast.",
                effect: StaticEffect::TaxOpponentSpellsTargeting { target_filter: yours(), amount: 2 },
            },
            StaticAbility {
                description: "Abilities your opponents activate that target a Merfolk you control cost {2} more to activate.",
                effect: StaticEffect::TaxOpponentAbilitiesTargeting { target_filter: yours(), amount: 2 },
            },
        ],
        ..legend(merfolk(
            "Kopala, Warden of Waves",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            2,
        ))
    }
}

/// Kumena, Tyrant of Orazca — tap one other Merfolk: it can't be blocked;
/// tap three: draw; tap five: a +1/+1 counter on each of your Merfolk.
pub fn kumena_tyrant_of_orazca() -> CardDefinition {
    let tap = |n, other: bool, effect| ActivatedAbility {
        tap_n_filter: Some((
            if other { R::Creature.and(is_merfolk()).and(R::OtherThanSource) } else { R::Creature.and(is_merfolk()) },
            n,
        )),
        effect,
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            tap(
                1,
                true,
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: crate::effect::Duration::EndOfTurn,
                },
            ),
            tap(3, false, Effect::Draw { who: Selector::You, amount: Value::ONE }),
            tap(
                5,
                false,
                Effect::AddCounter { what: your_merfolk(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ),
        ],
        ..legend(merfolk(
            "Kumena, Tyrant of Orazca",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Shaman],
            2,
            4,
        ))
    }
}

/// Mist Dancer — flying; other Merfolk you control get +1/+0 and flying;
/// encore {5}{U}{U}.
pub fn mist_dancer() -> CardDefinition {
    let others = || Selector::EachPermanent(R::Creature.and(is_merfolk()).and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Other Merfolk you control get +1/+0.",
                effect: StaticEffect::PumpPT { applies_to: others(), power: 1, toughness: 0 },
            },
            StaticAbility {
                description: "Other Merfolk you control have flying.",
                effect: StaticEffect::GrantKeyword { applies_to: others(), keyword: Keyword::Flying },
            },
        ],
        activated_abilities: vec![encore(cost(&[generic(5), u(), u()]))],
        ..merfolk("Mist Dancer", cost(&[generic(4), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 3, 3)
    }
}

/// Sage of Fables — your other Wizards enter with an extra +1/+1 counter; {2},
/// remove a +1/+1 counter from a creature you control: draw.
pub fn sage_of_fables() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each other Wizard creature you control enters with an additional +1/+1 counter on it.",
            effect: StaticEffect::TypedCreaturesEnterWithExtraCounter {
                types: vec![CreatureType::Wizard],
                kind: CounterType::PlusOnePlusOne,
                amount: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            remove_counter_among_filter: Some((
                Some(CounterType::PlusOnePlusOne),
                1,
                R::Creature.and(R::ControlledByYou),
            )),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..merfolk("Sage of Fables", cost(&[generic(2), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 2)
    }
}

/// Seafloor Oracle — a Merfolk of yours hitting a player draws a card.
pub fn seafloor_oracle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: is_merfolk() },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..merfolk("Seafloor Oracle", cost(&[generic(2), u(), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 3)
    }
}

/// Singer of Swift Rivers — flash; a shield counter on another creature of
/// yours; your Merfolk spells have flash.
pub fn singer_of_swift_rivers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            kind: CounterType::Shield,
            amount: Value::ONE,
        })],
        static_abilities: vec![StaticAbility {
            description: "You may cast Merfolk spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: is_merfolk() },
        }],
        ..merfolk(
            "Singer of Swift Rivers",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Shaman],
            3,
            2,
        )
    }
}

/// Surgespanner — becoming tapped, you may pay {1}{U} to bounce target
/// permanent.
pub fn surgespanner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {1}{U} to return target permanent to its owner's hand?".into(),
                mana_cost: cost(&[generic(1), u()]),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Permanent),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
                else_: None,
            },
        }],
        ..merfolk("Surgespanner", cost(&[generic(2), u(), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 2)
    }
}

/// Tishana, Voice of Thunder — */* equal to your hand size; no maximum hand
/// size; entering draws a card per creature you control.
pub fn tishana_voice_of_thunder() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ControllerHandSize),
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)))),
        })],
        ..legend(merfolk(
            "Tishana, Voice of Thunder",
            cost(&[generic(5), g(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Shaman],
            0,
            0,
        ))
    }
}

/// Topography Tracker — enters with a Map token; your creatures explore twice
/// whenever they would explore.
pub fn topography_tracker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(crabomination_base::tokens::map_token()),
        })],
        static_abilities: vec![StaticAbility {
            description: "If a creature you control would explore, instead it explores, then it explores again.",
            effect: StaticEffect::ExploresTwice,
        }],
        ..merfolk("Topography Tracker", cost(&[generic(2), g()]), vec![CreatureType::Merfolk, CreatureType::Scout], 2, 2)
    }
}

/// Tributary Instructor — mentor; a creature of yours with a +1/+1 counter
/// dying draws a card.
pub fn tributary_instructor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            mentor(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::WithCounter(CounterType::PlusOnePlusOne),
                    },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..merfolk(
            "Tributary Instructor",
            cost(&[generic(3), g()]),
            vec![CreatureType::Merfolk, CreatureType::Shaman],
            4,
            4,
        )
    }
}

/// Wave Goodbye — bounce each creature without a +1/+1 counter.
pub fn wave_goodbye() -> CardDefinition {
    CardDefinition {
        name: "Wave Goodbye",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::WithCounter(CounterType::PlusOnePlusOne))))),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// Xolatoyac, the Smiling Flood — entering or attacking puts a flood counter
/// on target land (an Island while it has one); at your end step, untap each
/// permanent you control with a counter. ⚠ The Island type ends with
/// Xolatoyac.
pub fn xolatoyac_the_smiling_flood() -> CardDefinition {
    let flood = || Effect::AddCounter { what: target_filtered(R::Land), kind: CounterType::Flood, amount: Value::ONE };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each land with a flood counter on it is an Island in addition to its other types.",
            effect: StaticEffect::LandTypeChanger {
                applies_to: Selector::EachPermanent(R::Land.and(R::WithCounter(CounterType::Flood))),
                land_type: LandType::Island,
                replace: false,
            },
        }],
        triggered_abilities: vec![
            etb(flood()),
            on_attack(flood()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::Untap {
                    what: Selector::EachPermanent(R::Permanent.and(R::ControlledByYou).and(R::WithAnyCounter)),
                    up_to: None,
                },
            },
        ],
        ..legend(merfolk(
            "Xolatoyac, the Smiling Flood",
            cost(&[generic(4), g(), u()]),
            vec![CreatureType::Salamander, CreatureType::Serpent],
            6,
            6,
        ))
    }
}
