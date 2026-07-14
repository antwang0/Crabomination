//! Modern Horizons 2 sweep, batch 4 — commons/uncommons + a few rares on
//! existing primitives (split cards, unearth, living weapon, landcycling,
//! threshold/metalcraft, exploit). Tests in `tests/mh2c.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement, Selector, SplitCard, SplitHalf, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    dies_ping_any, draw, etb, exploit, on_dies, target_any, target_filtered, unearth,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Predicate, StaticEffect, ZoneDest, ZoneRef,
};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color};

use SelectionRequirement as R;

fn thopter_token() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

fn basic_landcycling(c: crate::mana::ManaCost) -> Keyword {
    Keyword::Typecycling(Box::new((c, R::IsBasicLand)))
}

/// Aeromoeba — {3}{U} 2/4 flying. Discard a card: switch its P/T this turn.
pub fn aeromoeba() -> CardDefinition {
    CardDefinition {
        name: "Aeromoeba",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::SwitchPT { what: Selector::This, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Archfiend of Sorrows — {5}{B}{B} 4/5 flying. ETB opponents' creatures get
/// -2/-2 this turn; unearth {3}{B}{B}.
pub fn archfiend_of_sorrows() -> CardDefinition {
    CardDefinition {
        name: "Archfiend of Sorrows",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::EachMatching {
                zone: ZoneRef::Battlefield,
                filter: R::Creature.and(R::ControlledByOpponent),
            },
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![unearth(cost(&[generic(3), b(), b()]))],
        ..Default::default()
    }
}

/// Batterbone — {2} living-weapon Equipment; +1/+1, vigilance, lifelink;
/// equip {5}.
pub fn batterbone() -> CardDefinition {
    let germ = TokenDefinition {
        name: "Phyrexian Germ".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Batterbone",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(5)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: germ },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        ..Default::default()
    }
}

/// Battle Plan — {3}{R} enchantment. Begin combat on your turn: target
/// creature you control gets +2/+0 this turn. Basic landcycling {1}{R}.
pub fn battle_plan() -> CardDefinition {
    CardDefinition {
        name: "Battle Plan",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![basic_landcycling(cost(&[generic(1), r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Blacksmith's Skill — {W} instant. Target permanent gains hexproof and
/// indestructible this turn; +2/+2 too if it's an artifact creature.
pub fn blacksmiths_skill() -> CardDefinition {
    CardDefinition {
        name: "Blacksmith's Skill",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Permanent),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::Artifact.and(R::Creature),
                },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Blessed Respite — {1}{G} instant. Target player shuffles their graveyard
/// into their library; fog this turn.
pub fn blessed_respite() -> CardDefinition {
    CardDefinition {
        name: "Blessed Respite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::Target(0) },
            Effect::PreventAllCombatDamageThisTurn,
        ]),
        ..Default::default()
    }
}

/// Bottle Golems — {4} 3/3 trample. Dies: gain life equal to its power.
pub fn bottle_golems() -> CardDefinition {
    CardDefinition {
        name: "Bottle Golems",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_dies(Effect::GainLife {
            who: Selector::You,
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        ..Default::default()
    }
}

/// Cabal Initiate — {1}{B} 2/1. Discard a card: lifelink this turn.
/// Threshold — +1/+2.
pub fn cabal_initiate() -> CardDefinition {
    CardDefinition {
        name: "Cabal Initiate",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Threshold — This creature gets +1/+2 as long as there are seven or more cards in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ThresholdActive { who: PlayerRef::You },
                power: 1,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Clattering Augur — {1}{B} 1/1, can't block. ETB draw a card and lose 1
/// life. {2}{B}{B}: return from your graveyard to your hand.
pub fn clattering_augur() -> CardDefinition {
    CardDefinition {
        name: "Clattering Augur",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(1),
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), b()]),
            from_graveyard: true,
            sorcery_speed: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Crack Open — {2}{G} sorcery. Destroy target artifact or enchantment;
/// create a Treasure.
pub fn crack_open() -> CardDefinition {
    CardDefinition {
        name: "Crack Open",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            crate::effect::shortcut::mint_treasures(1),
        ]),
        ..Default::default()
    }
}

/// Etherium Spinner — {2}{U} 2/1. Cast a spell with mana value 4+: create a
/// 1/1 Thopter with flying.
pub fn etherium_spinner() -> CardDefinition {
    CardDefinition {
        name: "Etherium Spinner",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::ManaValueAtLeast(4))),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: thopter_token(),
            },
        }],
        ..Default::default()
    }
}

/// Fast // Furious — {2}{R} instant: discard a card, then draw two //
/// {3}{R}{R} sorcery: 3 damage to each creature without flying.
pub fn fast_furious() -> CardDefinition {
    CardDefinition {
        name: "Fast // Furious",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            draw(2),
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), r(), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::DealDamage {
                    amount: Value::Const(3),
                    to: Selector::EachMatching {
                        zone: ZoneRef::Battlefield,
                        filter: R::Creature.and(R::HasKeyword(Keyword::Flying).negate()),
                    },
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Feast of Sanity — {3}{B} enchantment. Whenever you discard a card, this
/// deals 1 damage to any target and you gain 1 life.
pub fn feast_of_sanity() -> CardDefinition {
    CardDefinition {
        name: "Feast of Sanity",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::DealDamage { amount: Value::ONE, to: target_any() },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Filigree Attendant — {2}{U}{U} */3 flying; power = artifacts you control.
pub fn filigree_attendant() -> CardDefinition {
    CardDefinition {
        name: "Filigree Attendant",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Homunculus],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower { base_p: 0, base_t: 3 }),
        ..Default::default()
    }
}

/// Flame Blitz — {R} enchantment. Your end step: 5 damage to each
/// planeswalker. Cycling {2}.
pub fn flame_blitz() -> CardDefinition {
    CardDefinition {
        name: "Flame Blitz",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::DealDamage {
                amount: Value::Const(5),
                to: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Planeswalker,
                },
            },
        }],
        ..Default::default()
    }
}

/// Fodder Tosser — {3} artifact. {T}, Discard a card: 2 damage to target
/// player or planeswalker.
pub fn fodder_tosser() -> CardDefinition {
    CardDefinition {
        name: "Fodder Tosser",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::DealDamage {
                amount: Value::Const(2),
                to: target_filtered(R::Player.or(R::Planeswalker)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Foundation Breaker — {3}{G} 2/2. ETB you may destroy target artifact or
/// enchantment. Evoke {1}{G}.
pub fn foundation_breaker() -> CardDefinition {
    CardDefinition {
        name: "Foundation Breaker",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 2,
        toughness: 2,
        alternative_cost: Some(crate::effect::shortcut::evoke(cost(&[generic(1), g()]))),
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "destroy target artifact or enchantment".into(),
            body: Box::new(Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment)),
            }),
        })],
        ..Default::default()
    }
}

/// Landscaper Colos — {5}{W} 4/6. ETB put target card from an opponent's
/// graveyard on the bottom of their library. Basic landcycling {1}{W}.
pub fn landscaper_colos() -> CardDefinition {
    CardDefinition {
        name: "Landscaper Colos",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goat, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![basic_landcycling(cost(&[generic(1), w()]))],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::OwnedByYou.negate()),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: crate::effect::LibraryPosition::Bottom,
            },
        })],
        ..Default::default()
    }
}

/// Lightning Spear — {1}{R} Equipment. +1/+0 and trample; {2}{R}, sacrifice:
/// 3 damage to any target. Equip {1}.
pub fn lightning_spear() -> CardDefinition {
    CardDefinition {
        name: "Lightning Spear",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_cost: true,
            effect: Effect::DealDamage { amount: Value::Const(3), to: target_any() },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Loathsome Curator — {4}{B} 5/4 menace, exploit. On exploit: destroy target
/// creature you don't control with mana value 3 or less.
pub fn loathsome_curator() -> CardDefinition {
    CardDefinition {
        name: "Loathsome Curator",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gorgon, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![exploit(Effect::Destroy {
            what: target_filtered(
                R::Creature.and(R::ControlledByOpponent).and(R::ManaValueAtMost(3)),
            ),
        })],
        ..Default::default()
    }
}

/// Moderation — {1}{W}{U} enchantment. You can't cast more than one spell
/// each turn; whenever you cast a spell, draw a card.
pub fn moderation() -> CardDefinition {
    CardDefinition {
        name: "Moderation",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You can't cast more than one spell each turn.",
            effect: StaticEffect::OneSpellPerTurn,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: draw(1),
        }],
        ..Default::default()
    }
}

/// Monoskelion — {2} 1/1; enters with a +1/+1 counter. {1}, remove a +1/+1
/// counter: 1 damage to any target.
pub fn monoskelion() -> CardDefinition {
    CardDefinition {
        name: "Monoskelion",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 1,
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::DealDamage { amount: Value::ONE, to: target_any() },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Parcel Myr — {1}{U} 2/1. {2}, Sacrifice: draw a card.
pub fn parcel_myr() -> CardDefinition {
    CardDefinition {
        name: "Parcel Myr",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Myr],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Piru, the Volatile — {2}{R}{R}{W}{W}{B}{B} 7/7 flying, lifelink. Upkeep:
/// sacrifice unless you pay {R}{W}{B}. Dies: 7 damage to each nonlegendary
/// creature.
pub fn piru_the_volatile() -> CardDefinition {
    CardDefinition {
        name: "Piru, the Volatile",
        cost: cost(&[generic(2), r(), r(), w(), w(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dragon],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::MayPay {
                    description: "Pay {R}{W}{B} or sacrifice Piru?".into(),
                    mana_cost: cost(&[r(), w(), b()]),
                    body: Box::new(Effect::Noop),
                    else_: Some(Box::new(Effect::SacrificeSource)),
                },
            },
            on_dies(Effect::DealDamage {
                amount: Value::Const(7),
                to: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::HasSupertype(Supertype::Legendary).negate()),
                },
            }),
        ],
        ..Default::default()
    }
}

/// Rishadan Dockhand — {U} 1/2 islandwalk. {1}, {T}: tap target land.
pub fn rishadan_dockhand() -> CardDefinition {
    CardDefinition {
        name: "Rishadan Dockhand",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Land) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Said // Done — {2}{U} sorcery: return target instant or sorcery card from
/// your graveyard to your hand // {3}{U} instant: tap up to two target
/// creatures; they don't untap during their controller's next untap step.
pub fn said_done() -> CardDefinition {
    CardDefinition {
        name: "Said // Done",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), u()]),
                card_types: vec![CardType::Instant],
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: Selector::Target(0) },
                        Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::Stun,
                            amount: Value::ONE,
                        },
                    ])),
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Slag Strider — {5}{R}{R} 3/3, affinity for artifacts. {1}, Sacrifice an
/// artifact: 1 damage to any target.
pub fn slag_strider() -> CardDefinition {
    CardDefinition {
        name: "Slag Strider",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 3,
        toughness: 3,
        affinity_filter: Some(R::Artifact),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::DealDamage { amount: Value::ONE, to: target_any() },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Storm God's Oracle — {1}{U}{R} 1/3. {1}: +1/-1 this turn. Dies: 3 damage
/// to any target.
pub fn storm_gods_oracle() -> CardDefinition {
    CardDefinition {
        name: "Storm God's Oracle",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![dies_ping_any(3)],
        ..Default::default()
    }
}

/// Vedalken Infiltrator — {1}{U} 1/3, can't be blocked. Metalcraft — +1/+0.
pub fn vedalken_infiltrator() -> CardDefinition {
    CardDefinition {
        name: "Vedalken Infiltrator",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Unblockable],
        static_abilities: vec![StaticAbility {
            description: "Metalcraft — This creature gets +1/+0 as long as you control three or more artifacts.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::MetalcraftActive { who: PlayerRef::You },
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Viashino Lashclaw — {1}{R} 2/2. {T}, Discard a card: creatures you control
/// gain haste this turn.
pub fn viashino_lashclaw() -> CardDefinition {
    CardDefinition {
        name: "Viashino Lashclaw",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// World-Weary — {3}{B}{B} Aura: enchanted creature gets -4/-4. Basic
/// landcycling {1}{B}.
pub fn world_weary() -> CardDefinition {
    CardDefinition {
        name: "World-Weary",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![basic_landcycling(cost(&[generic(1), b()]))],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: -4, toughness: -4, ..Default::default() }),
        ..Default::default()
    }
}
