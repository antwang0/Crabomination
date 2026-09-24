//! Commander: the cards the **Silverquill Influence** precon (SOC, Killian,
//! Decisive Mentor) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_killian.rs`.
//!
//! Residuals (each also on its card):
//! - **Armored Skyhunter** — an Equipment it puts onto the battlefield stays
//!   unattached.
//! - **Coercive Impetus** — the goad is renewed at the beginning of each
//!   combat by a trigger, not a static.
//! - **Herald of Amity** — the eight cards are revealed rather than exiled.
//! - **Intermediate Chirography** — level 3 counts creatures that died with
//!   counters on them; an Aura or Equipment alone doesn't make one modified.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, w, Color, ManaCost};

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

fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn inkling() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Inkling".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Inkling], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    })
}

fn make_inkling() -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: inkling() }
}

fn your_auras() -> Selector {
    Selector::EachPermanent(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).and(R::ControlledByYou))
}

fn become_prepared() -> Effect {
    Effect::AddCounterCapped { what: Selector::This, kind: CounterType::Prepared, amount: Value::ONE, cap: Value::ONE }
}

fn plains_swamp(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Plains, LandType::Swamp], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add(Color::White), crate::sets::tap_add(Color::Black)],
        ..Default::default()
    }
}

/// Killian, Decisive Mentor — an enchantment of yours entering taps and goads
/// up to one target creature; one or more creatures enchanted by your Auras
/// attacking draws you a card.
pub fn killian_decisive_mentor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
                ),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                        Effect::Goad { what: Selector::Target(0) },
                    ])),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::EnchantedByYourAura,
                    })
                    .once_per_batch(),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..creature(
            "Killian, Decisive Mentor",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            2,
            3,
        )
    }
}

/// Armored Skyhunter — flying; attacking looks at the top six for an Aura or
/// Equipment to put onto the battlefield. Residual: an Equipment isn't
/// attached.
pub fn armored_skyhunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::LookTopPutMatchingOntoBattlefield {
            count: Value::Const(6),
            filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
            then: None,
            max: Some(1),
            tapped: false,
            exile_rest: false,
            rest_to_graveyard: false,
        })],
        ..creature("Armored Skyhunter", cost(&[generic(3), w()]), vec![CreatureType::Cat, CreatureType::Knight], 3, 3)
    }
}

/// Chains of Custody — enchant creature you control; entering exiles an
/// opponent's nonland permanent until it leaves; the host has ward {2}.
pub fn chains_of_custody() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Nonland.and(R::ControlledByOpponent)),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has ward {2}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: host(),
                keyword: Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
            },
        }],
        ..aura("Chains of Custody", cost(&[generic(2), w()]), R::Creature.and(R::ControlledByYou))
    }
}

/// Changing Loyalty — flash, replicate {2}; when the enchanted creature dies,
/// it returns under your control.
pub fn changing_loyalty() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Replicate(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..aura("Changing Loyalty", cost(&[generic(1), b()]), R::Creature)
    }
}

/// Coercive Impetus — enchanted creature gets +1/+1 and is goaded; when it
/// attacks, you draw a card and lose 1 life.
pub fn coercive_impetus() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: host(), power: 1, toughness: 1 },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
                effect: Effect::Goad { what: host() },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                ]),
            },
        ],
        ..aura("Coercive Impetus", cost(&[generic(2), b()]), R::Creature)
    }
}

/// Defacing Duskmage // Vandal's Edit — deathtouch; an opponent's second draw
/// each turn prepares it. Vandal's Edit: draw two, each player loses 2.
pub fn defacing_duskmage() -> CardDefinition {
    let vandals_edit = CardDefinition {
        name: "Vandal's Edit",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
        ]),
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl).with_filter(
                Predicate::ValueEquals(Value::CardsDrawnThisTurn(PlayerRef::Triggerer), Value::Const(2)),
            ),
            effect: become_prepared(),
        }],
        prepare_spell: Some(Arc::new(vandals_edit)),
        ..creature("Defacing Duskmage", cost(&[w(), b()]), vec![CreatureType::Dog, CreatureType::Warlock], 2, 2)
    }
}

/// Eiganjo Dynastorian // Replenish — vigilance; attacking with two or more
/// creatures prepares it. Replenish returns every enchantment card from your
/// graveyard.
pub fn eiganjo_dynastorian() -> CardDefinition {
    let replenish = CardDefinition {
        name: "Replenish",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ReturnAllMatchingFromGraveyardToBattlefield {
            who: PlayerRef::You,
            filter: R::Enchantment,
            sacrifice_eot: false,
        },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::ControlledByYou)),
                    n: Value::Const(2),
                },
            ),
            effect: become_prepared(),
        }],
        prepare_spell: Some(Arc::new(replenish)),
        ..creature("Eiganjo Dynastorian", cost(&[generic(2), w()]), vec![CreatureType::Fox, CreatureType::Advisor], 2, 3)
    }
}

/// Eriette of the Charmed Apple — creatures enchanted by your Auras can't
/// attack you or your planeswalkers; your end step drains each opponent for
/// your Aura count.
pub fn eriette_of_the_charmed_apple() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Each creature enchanted by an Aura you control can't attack you or planeswalkers you control.",
            effect: StaticEffect::CreaturesCantAttackController {
                protect_planeswalkers: true,
                filter: Some(R::EnchantedByYourAura),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::CountOf(Box::new(your_auras())),
            },
        }],
        ..creature(
            "Eriette of the Charmed Apple",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            2,
            4,
        )
    }
}

/// Forum Filibuster — your upkeep makes a 2/1 flying Inkling, then returns up
/// to one Aura or Equipment card from your graveyard attached to it.
pub fn forum_filibuster() -> CardDefinition {
    CardDefinition {
        name: "Forum Filibuster",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                make_inkling(),
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::AttachAuraFromGraveyardTo {
                        aura: target_filtered(
                            R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                                .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment))
                                .from_your_graveyard(),
                        ),
                        host: Selector::LastCreatedToken,
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Herald of Amity — flying; entering may cast an Aura from the top eight
/// free; attacking pumps it by your Aura count. Residual: revealed, not exiled.
pub fn herald_of_amity() -> CardDefinition {
    let auras = Value::CountOf(Box::new(your_auras()));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::RevealTopMayCastOneFree {
                count: Value::Const(8),
                max_mv: Value::Const(99),
                filter: Some(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
            }),
            on_attack(Effect::PumpPT {
                what: Selector::This,
                power: auras.clone(),
                toughness: auras,
                duration: Duration::EndOfTurn,
            }),
        ],
        ..creature("Herald of Amity", cost(&[generic(3), w()]), vec![CreatureType::Griffin], 2, 2)
    }
}

/// Intermediate Chirography — Class. Enters with a 2/1 Inkling. L2 ({1}{B}):
/// your first life loss each turn puts a +1/+1 counter on a creature of yours.
/// L3 ({2}{B}): each end step after a modified creature of yours died makes
/// an Inkling. Residual: "modified" is read as "had counters".
pub fn intermediate_chirography() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    CardDefinition {
        name: "Intermediate Chirography",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        triggered_abilities: vec![
            etb(make_inkling()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeLost, EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(2))
                    .once_per_turn(),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                    Predicate::All(vec![
                        Predicate::SourceClassLevelAtLeast(3),
                        Predicate::ValueAtLeast(
                            Value::CreatureDeathsThisTurnMatching {
                                filter: R::WithAnyCounter.and(R::ControlledByYou),
                            },
                            Value::ONE,
                        ),
                    ]),
                ),
                effect: make_inkling(),
            },
        ],
        activated_abilities: vec![
            level_up(cost(&[generic(1), b()]), 1),
            level_up(cost(&[generic(2), b()]), 2),
        ],
        ..Default::default()
    }
}

/// Pearl-Ear, Imperial Advisor — lifelink; your enchantment spells have
/// affinity for Auras; an Aura spell targeting a modified permanent of yours
/// draws a card.
pub fn pearl_ear_imperial_advisor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Enchantment spells you cast have affinity for Auras.",
            effect: StaticEffect::CostReductionByValue {
                filter: R::Enchantment,
                amount: Value::CountOf(Box::new(your_auras())),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).and(
                    R::SpellTargetsMatching(Box::new(R::IsModified.and(R::ControlledByYou))),
                )),
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Pearl-Ear, Imperial Advisor",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Fox, CreatureType::Advisor],
            3,
            4,
        )
    }
}

/// Raffine's Guidance — enchanted creature gets +1/+1; castable from your
/// graveyard for {2}{W}.
pub fn raffines_guidance() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: host(), power: 1, toughness: 1 },
        }],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), w()]),
            from_graveyard: true,
            ..Default::default()
        }),
        ..aura("Raffine's Guidance", cost(&[w()]), R::Creature)
    }
}

/// Scriv, the Obligator — flying, deathtouch; entering or attacking attaches a
/// Contract Aura token to an opponent's creature: when it attacks, +2/+0 if it
/// attacks one of your opponents, otherwise its controller loses 2.
pub fn scriv_the_obligator() -> CardDefinition {
    let contract = TokenDefinition {
        name: "Contract".into(),
        card_types: vec![CardType::Enchantment],
        colors: vec![Color::White],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource),
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: host(), filter: R::IsAttackingAnOpponent },
                then: Box::new(Effect::PumpPT {
                    what: host(),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(host()))),
                    amount: Value::Const(2),
                }),
            },
        }],
        ..Default::default()
    };
    let attach = || Effect::CreateTokenAttachedTo {
        target: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        definition: Arc::new(contract.clone()),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![etb(attach()), on_attack(attach())],
        ..creature(
            "Scriv, the Obligator",
            cost(&[generic(2), w(), b()]),
            vec![CreatureType::Inkling, CreatureType::Bird],
            2,
            3,
        )
    }
}

/// Sunlit Marsh — Plains Swamp; enters tapped.
pub fn sunlit_marsh() -> CardDefinition {
    CardDefinition { static_abilities: vec![crate::sets::enters_tapped()], ..plains_swamp("Sunlit Marsh") }
}

/// Turbulent Moor — Plains Swamp; enters tapped unless your opponents control
/// eight or more lands.
pub fn turbulent_moor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless your opponents control eight or more lands.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                    n: Value::Const(8),
                },
            },
        }],
        ..plains_swamp("Turbulent Moor")
    }
}

/// Umbral Expanse — Plains Swamp; enters tapped; cycling {2}.
pub fn umbral_expanse() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..plains_swamp("Umbral Expanse")
    }
}
