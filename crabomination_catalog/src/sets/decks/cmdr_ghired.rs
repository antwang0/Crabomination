//! Commander: the cards the **Primal Genesis** precon (C19, Ghired, Conclave
//! Exile) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Cliffside Rescuer** — protection from each opponent is protection from
//!   what opponents control (`ProtectionFromMatching(ControlledByOpponent)`).
//! - **Tahngarth, First Mate** — it attacks its new controller's default
//!   opponent, not a chosen player that opponent is attacking.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, w, x};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn populate() -> Effect {
    Effect::Populate { who: PlayerRef::You }
}

fn copy_of(source: Selector) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        extra_keywords: vec![],
    }
}

/// Ghired, Conclave Exile — a 4/4 trampling Rhino on entry; each attack
/// populates, the copy entering tapped and attacking.
pub fn ghired_conclave_exile() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Rhino".into(),
                    power: 4,
                    toughness: 4,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Rhino],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Trample],
                    ..Default::default()
                }),
            }),
            on_attack(Effect::Seq(vec![
                populate(),
                Effect::JoinCombatAttacking { what: Selector::LastCreatedTokens },
            ])),
        ],
        ..legendary(creature(
            "Ghired, Conclave Exile",
            cost(&[generic(2), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            2,
            5,
        ))
    }
}

/// Atla Palani, Nest Tender — {2}, {T}: a 0/1 defender Egg; an Egg of yours
/// dying reveals until a creature card and puts it onto the battlefield.
pub fn atla_palani_nest_tender() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Egg".into(),
                    power: 0,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Egg],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Defender],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Egg),
                },
            ),
            effect: Effect::RevealUntilOneToBattlefieldRestBottom {
                filter: R::Creature,
                damage_controller: false,
            },
        }],
        ..legendary(creature(
            "Atla Palani, Nest Tender",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            2,
            3,
        ))
    }
}

/// Cliffside Rescuer — vigilance; {T}, sacrifice: a permanent of yours gains
/// protection from your opponents until end of turn.
pub fn cliffside_rescuer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                keyword: Keyword::ProtectionFromMatching(Box::new(R::ControlledByOpponent)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Cliffside Rescuer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Commander's Insignia — +1/+1 to your creatures per command-zone cast of
/// your commander.
pub fn commanders_insignia() -> CardDefinition {
    let v = || Value::CommanderCastsFromCommandZone(PlayerRef::You);
    CardDefinition {
        name: "Commander's Insignia",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1 for each time you've cast your commander from the command zone this game.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: v(),
                toughness: v(),
            },
        }],
        ..Default::default()
    }
}

/// Doomed Artisan — your Sculptures can't attack or block; each of your end
/// steps makes a Sculpture as big as your Sculpture count.
pub fn doomed_artisan() -> CardDefinition {
    let sculptures = || R::HasCreatureType(CreatureType::Sculpture).and(R::ControlledByYou);
    let sculpture = TokenDefinition {
        name: "Sculpture".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sculpture],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "This token's power and toughness are each equal to the number of Sculptures you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: sculptures(),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Sculptures you control can't attack.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(sculptures()),
                    keyword: Keyword::CantAttack,
                },
            },
            StaticAbility {
                description: "Sculptures you control can't block.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(sculptures()),
                    keyword: Keyword::CantBlock,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(sculpture),
            },
        }],
        ..creature(
            "Doomed Artisan",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            1,
        )
    }
}

/// Full Flowering — populate X times.
pub fn full_flowering() -> CardDefinition {
    CardDefinition {
        name: "Full Flowering",
        cost: cost(&[x(), x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Repeat { count: Value::XFromCost, body: Box::new(populate()) },
        ..Default::default()
    }
}

/// Ghired's Belligerence — X damage divided among creatures; each one that
/// dies this turn populates.
pub fn ghireds_belligerence() -> CardDefinition {
    const SLOTS: usize = 5;
    let mut body: Vec<Effect> = (0..SLOTS)
        .map(|slot| Effect::WhenTargetDiesThisTurn { body: Box::new(populate()), slot, filter: None })
        .collect();
    body.push(Effect::DealDamageDivided {
        total: Value::XFromCost,
        filter: R::Creature,
        max_targets: SLOTS as u8,
        retaliate_to_source: false,
    });
    CardDefinition {
        name: "Ghired's Belligerence",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(body),
        ..Default::default()
    }
}

/// Marisi, Breaker of the Coil — opponents can't cast spells during combat;
/// combat damage to a player goads everything that player controls.
pub fn marisi_breaker_of_the_coil() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't cast spells during combat.",
            effect: StaticEffect::OpponentsCantCastDuringCombat,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::Goad {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByTriggerPlayer)),
            },
        }],
        ..legendary(creature(
            "Marisi, Breaker of the Coil",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            5,
            4,
        ))
    }
}

/// Mimic Vat — a dying nontoken creature may be imprinted (the old one goes
/// to the graveyard); {3}, {T}: a hasty token copy, exiled at end step.
pub fn mimic_vat() -> CardDefinition {
    CardDefinition {
        name: "Mimic Vat",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
            ),
            effect: Effect::MayDo {
                description: "Exile the dying creature with Mimic Vat?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::CardExiledWithSource, to: ZoneDest::Graveyard },
                    Effect::ExileWithSource { what: Selector::TriggerSource },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                copy_of(Selector::CardExiledWithSource),
                Effect::GrantKeyword {
                    what: Selector::LastCreatedToken,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::ExileLastCreatedTokensAtNextEndStep,
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Second Harvest — copy each token you control.
pub fn second_harvest() -> CardDefinition {
    CardDefinition {
        name: "Second Harvest",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::IsToken.and(R::ControlledByYou)),
            body: Box::new(copy_of(Selector::TriggerSource)),
        },
        ..Default::default()
    }
}

/// Selesnya Eulogist — {2}{G}: exile a creature card from a graveyard, then
/// populate.
pub fn selesnya_eulogist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.from_any_graveyard()),
                    to: ZoneDest::Exile,
                },
                populate(),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Selesnya Eulogist",
            cost(&[generic(2), g()]),
            vec![CreatureType::Centaur, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Slice in Twain — destroy an artifact or enchantment; draw a card.
pub fn slice_in_twain() -> CardDefinition {
    CardDefinition {
        name: "Slice in Twain",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Song of the Worldsoul — whenever you cast a spell, populate.
pub fn song_of_the_worldsoul() -> CardDefinition {
    CardDefinition {
        name: "Song of the Worldsoul",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: populate(),
        }],
        ..Default::default()
    }
}

/// Tahngarth, First Mate — blocked by at most one creature; while tapped, an
/// attacking opponent may borrow it until end of combat, attacking.
pub fn tahngarth_first_mate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Tapped }),
            effect: Effect::MayDo {
                description: "Let the attacking opponent gain control of Tahngarth this combat?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::GainControl {
                        what: Selector::This,
                        to: Some(PlayerRef::ActivePlayer),
                        duration: Duration::EndOfCombat,
                    },
                    Effect::JoinCombatAttacking { what: Selector::This },
                ])),
            },
        }],
        ..legendary(creature(
            "Tahngarth, First Mate",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            5,
            5,
        ))
    }
}

/// Tectonic Hellion — haste; its attack makes each player with the most
/// lands sacrifice two.
pub fn tectonic_hellion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::PlayersWithMostSacrifice {
            filter: R::Land,
            count: Value::Const(2),
        })],
        ..creature("Tectonic Hellion", cost(&[generic(5), r(), r()]), vec![CreatureType::Hellion], 8, 5)
    }
}

/// Voice of Many — draw a card per opponent with fewer creatures than you.
pub fn voice_of_many() -> CardDefinition {
    let creatures_of = |who: PlayerRef| {
        Value::CountOf(Box::new(Selector::ControlledBy { who, filter: R::Creature }))
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ForEachOpponent {
            body: Box::new(Effect::If {
                cond: Predicate::ValueAtLeast(
                    creatures_of(PlayerRef::You),
                    Value::Sum(vec![creatures_of(PlayerRef::Triggerer), Value::ONE]),
                ),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            }),
        })],
        ..creature(
            "Voice of Many",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            3,
        )
    }
}
