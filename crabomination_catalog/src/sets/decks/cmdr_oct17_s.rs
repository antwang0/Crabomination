//! Commander: 2026-10-17 (`3chree`) closers, third batch — one-deck closers
//! from the EDHREC long tail. Tests in `tests/recent_b/cmdr_oct17_s.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{draw, etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, g, generic, r, snow_mana, u, w};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}




/// Mirror-Sigil Sergeant — {5}{W} 4/4 trample Rhino Soldier. Your upkeep, if
/// you control a blue permanent: you may create a token copy of it.
pub fn mirror_sigil_sergeant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(
                Predicate::SelectorExists(Selector::EachPermanent(R::HasColor(Color::Blue).and(R::ControlledByYou))),
            ),
            effect: Effect::MayDo {
                description: "Create a token that's a copy of Mirror-Sigil Sergeant?".into(),
                body: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                }),
            },
        }],
        ..creature(
            "Mirror-Sigil Sergeant",
            cost(&[generic(5), w()]),
            vec![CreatureType::Rhino, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Secrets of the Golden City — {1}{U}{U} sorcery. Ascend; draw two, or three
/// with the city's blessing.
pub fn secrets_of_the_golden_city() -> CardDefinition {
    spell(
        "Secrets of the Golden City",
        cost(&[generic(1), u(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Ascend { who: PlayerRef::You },
            Effect::If {
                cond: Predicate::HasCityBlessing { who: PlayerRef::You },
                then: Box::new(draw(3)),
                else_: Box::new(draw(2)),
            },
        ]),
    )
}

/// Magmatic Channeler — {1}{R} 1/3 Human Wizard; +3/+1 with 4+ instants and/or
/// sorceries in your graveyard. {T}, discard a card: exile top two, choose one;
/// you may play it this turn.
pub fn magmatic_channeler() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Gets +3/+1 while 4+ instant and/or sorcery cards are in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    },
                    Value::Const(4),
                ),
                power: 3,
                toughness: 1,
                keywords: vec![],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: crate::sets::impulse_choose_one(2, MayPlayDuration::EndOfThisTurn),
            ..Default::default()
        }],
        ..creature(
            "Magmatic Channeler",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Twinned Vision — {1}{U/R} instant. Draw one, or two if not cast from your
/// hand. Flashback—{1}{U/R}{U/R}, discard a card.
pub fn twinned_vision() -> CardDefinition {
    let ur = ManaSymbol::Hybrid(Color::Blue, Color::Red);
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), ur, ur]))],
        flashback_additional_cost: vec![AdditionalCastCost::Discard { count: 1, filter: None }],
        ..spell(
            "Twinned Vision",
            cost(&[generic(1), ur]),
            CardType::Instant,
            Effect::If { cond: Predicate::CastFromHand, then: Box::new(draw(1)), else_: Box::new(draw(2)) },
        )
    }
}

/// Choose Your Weapon — {2}{G} instant. Choose one — double target creature's
/// power and toughness until end of turn; or 5 damage to target flier.
pub fn choose_your_weapon() -> CardDefinition {
    spell(
        "Choose Your Weapon",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::PowerOf(Box::new(Selector::Target(0))),
                toughness: Value::ToughnessOf(Box::new(Selector::Target(0))),
                duration: Duration::EndOfTurn,
            },
            crate::effect::shortcut::deal(5, target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying)))),
        ]),
    )
}

/// Arcum's Astrolabe — {S} snow artifact. ETB: draw a card. {1}, {T}: add one
/// mana of any color.
pub fn arcums_astrolabe() -> CardDefinition {
    CardDefinition {
        name: "Arcum's Astrolabe",
        cost: cost(&[snow_mana()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(draw(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            ..crate::sets::tap_add_any_color()
        }],
        ..Default::default()
    }
}


/// Favor of Jukai — {3}{G} Aura, enchant artifact or creature. As a creature:
/// +3/+3 and reach. Channel — {1}{G}, discard: target creature +3/+3 and reach EOT.
pub fn favor_of_jukai() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Favor of Jukai",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.or(R::Artifact)) },
        static_abilities: vec![
            StaticAbility {
                description: "As long as enchanted permanent is a creature, it gets +3/+3",
                effect: StaticEffect::PumpPT { applies_to: enchanted(), power: 3, toughness: 3 },
            },
            StaticAbility {
                description: "and has reach.",
                effect: StaticEffect::GrantKeyword { applies_to: enchanted(), keyword: Keyword::Reach },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Reach,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Twisted Fates — {2}{W}{W}{B} sorcery. Destroy target nonland permanent; a
/// +1/+1 counter on each creature target player (slot 1) controls.
pub fn twisted_fates() -> CardDefinition {
    spell(
        "Twisted Fates",
        cost(&[generic(2), w(), w(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Nonland.and(R::Permanent)) },
            Effect::AddCounter {
                what: Selector::ControlledBy { who: PlayerRef::Target(1), filter: R::Creature },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
    )
}


/// Fateshaper Aspirant — {4}{W} 3/4 Rhino Cleric. ETB, choose one — return
/// target legendary card from your graveyard to hand; or a +1/+1 counter on
/// target creature, which gains vigilance and indestructible until EOT.
pub fn fateshaper_aspirant() -> CardDefinition {
    let tc = || target_filtered(R::Creature);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::HasSupertype(Supertype::Legendary).from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Seq(vec![
                Effect::AddCounter { what: tc(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::GrantKeyword { what: tc(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: tc(), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            ]),
        ]))],
        ..creature("Fateshaper Aspirant", cost(&[generic(4), w()]), vec![CreatureType::Rhino, CreatureType::Cleric], 3, 4)
    }
}

/// Snowslope Hunter — {2}{R} 2/3 Goblin Ranger. Sacrifice another creature or
/// artifact: impulse the top card until the end of your next turn. Your turn,
/// once each turn.
pub fn snowslope_hunter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.or(R::Artifact), 1)),
            once_per_turn: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary { who: Selector::You, amount: Value::ONE, link_to_source: false, face_down: false },
                Effect::GrantMayPlay {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Snowslope Hunter", cost(&[generic(2), r()]), vec![CreatureType::Goblin, CreatureType::Ranger], 2, 3)
    }
}

/// Singing Bell Strike — {1}{U} Aura. ETB taps the enchanted creature; it
/// doesn't untap during its controller's untap step and has "{6}: Untap this creature."
pub fn singing_bell_strike() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Singing Bell Strike",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Tap { what: enchanted() })],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature doesn't untap during its controller's untap step.",
                effect: StaticEffect::PreventUntap { applies_to: enchanted() },
            },
            StaticAbility {
                description: "Enchanted creature has \"{6}: Untap this creature.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: enchanted(),
                    ability: ActivatedAbility {
                        mana_cost: cost(&[generic(6)]),
                        effect: Effect::Untap { what: Selector::This, up_to: None },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Underhanded Designs — {1}{B} enchantment. An artifact of yours entering:
/// you may pay {1} to drain each opponent for 1. {1}{B}, sacrifice: destroy
/// target creature; only with two or more artifacts.
pub fn underhanded_designs() -> CardDefinition {
    CardDefinition {
        name: "Underhanded Designs",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: Effect::MayPay {
                description: "Pay {1} to drain each opponent for 1?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ])),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                n: Value::Const(2),
            }),
            effect: Effect::Destroy { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Last One Standing — {1}{B}{R} Sorcery. Choose a creature at random, then
/// destroy the rest (The Nipton Lottery's `ChooseOneAtRandomAmong` shape).
pub fn last_one_standing() -> CardDefinition {
    CardDefinition {
        name: "Last One Standing",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseOneAtRandomAmong {
            what: Selector::EachPermanent(R::Creature),
            chosen: Box::new(Effect::Noop),
            other: Box::new(Effect::Destroy { what: Selector::SeparatedPile { chosen: false } }),
        },
        ..Default::default()
    }
}



/// Rewrite Regrets — {3}{B} Sorcery. Reanimate target creature or
/// planeswalker card with mana value 6 or less. Empower Jace 2.
pub fn rewrite_regrets() -> CardDefinition {
    CardDefinition {
        name: "Rewrite Regrets",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    (R::Creature.or(R::Planeswalker)).and(R::InYourGraveyard).and(R::ManaValueAtMost(6)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Armguard Familiar — {1}{U} 2/1 Equipment Beast artifact creature, ward {2}.
/// Equipped creature gets +2/+1 and has ward {2}. Reconfigure {4}.
pub fn armguard_familiar() -> CardDefinition {
    CardDefinition {
        name: "Armguard Familiar",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Ward(WardCost::generic(2)), Keyword::Reconfigure(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 1,
            keywords: vec![Keyword::Ward(WardCost::generic(2))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Warrior's Blades — {2}{R}{W} Equipment. ETB: 3 damage to any target and you
/// gain 3 life. +2/+1. Equip {3}, {1} less per +1/+1 counter on the target
/// (three stacked `EquipCostReducedIfTarget` thresholds; {3} caps it).
pub fn warriors_blades() -> CardDefinition {
    let per_counter = |n: u32| StaticAbility {
        description: "This ability costs {1} less to activate for each +1/+1 counter on the creature it targets.",
        effect: StaticEffect::EquipCostReducedIfTarget {
            filter: R::WithCounterAtLeast(CounterType::PlusOnePlusOne, n),
            amount: 1,
        },
    };
    CardDefinition {
        name: "Warrior's Blades",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        static_abilities: vec![per_counter(1), per_counter(2), per_counter(3)],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]))],
        ..Default::default()
    }
}

/// Veilstone Amulet — {3} artifact. Whenever you cast a spell, creatures you
/// control can't be the targets of spells or abilities your opponents control
/// this turn.
pub fn veilstone_amulet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::YourCreaturesUntargetableByOpponentsThisTurn,
        }],
        ..spell("Veilstone Amulet", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Light from Within — {2}{W}{W} enchantment. Chroma — each creature you
/// control gets +1/+1 for each white mana symbol in its mana cost.
pub fn light_from_within() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Chroma — Each creature you control gets +1/+1 for each white mana symbol in its mana cost.",
            effect: StaticEffect::PumpTeamByControlledPermanents {
                applies_to: R::Creature,
                count_filter: R::Any,
                per_power: 1,
                per_toughness: 1,
                count_graveyard: false,
                exclude_self: false,
                per_own_color: false,
                per_own_counter: None,
                per_own_pips: Some(Color::White),
            },
        }],
        ..spell("Light from Within", cost(&[generic(2), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Corpse Explosion — {1}{B}{R} sorcery. As an additional cost, exile a
/// creature card from your graveyard; it deals damage equal to that card's
/// power to each creature and each planeswalker.
pub fn corpse_explosion() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::ExileFromGraveyardForPower { filter: R::Creature }],
        ..spell(
            "Corpse Explosion",
            cost(&[generic(1), b(), r()]),
            CardType::Sorcery,
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.or(R::Planeswalker)),
                amount: Value::XFromCost,
            },
        )
    }
}

/// Command the Stage — {2}{R} sorcery. Create a 2/2 Wizard Soldier "Cadet",
/// then a +1/+1 counter on each other Wizard token you control. Each upkeep,
/// if an opponent was dealt noncombat damage last turn, it returns from your
/// graveyard to your hand.
pub fn command_the_stage() -> CardDefinition {
    let cadet = std::sync::Arc::new(TokenDefinition {
        name: "Cadet".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        // Counters first, then the Cadet: "each other Wizard token".
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::FromYourGraveyardAnyPlayer)
                .with_filter(Predicate::OpponentDealtNoncombatDamageLastTurn),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
        }],
        ..spell(
            "Command the Stage",
            cost(&[generic(2), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::IsToken.and(R::HasCreatureType(CreatureType::Wizard)).and(R::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: cadet },
            ]),
        )
    }
}

/// Vesuvan Doppelganger — {3}{U}{U} 0/0 Shapeshifter. May enter as a copy of
/// any creature, except it keeps its own color and has "At the beginning of
/// your upkeep, you may have this become a copy of target creature, except it
/// doesn't copy that creature's color and it has this ability."
pub fn vesuvan_doppelganger() -> CardDefinition {
    let recopy = TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect: Effect::MayDo {
            description: "Have Vesuvan Doppelganger become a copy of target creature?".into(),
            body: Box::new(Effect::BecomeCopyKeepingColorAndThisAbility {
                what: Selector::This,
                source: target_filtered(R::Creature),
            }),
        },
    };
    CardDefinition {
        enters_as_copy: Some(crate::card::EntersAsCopy {
            filter: R::Creature.and(R::OtherThanSource),
            keep_own_colors: true,
            extra_triggered: vec![recopy],
            ..Default::default()
        }),
        ..creature("Vesuvan Doppelganger", cost(&[generic(3), u(), u()]), vec![CreatureType::Shapeshifter], 0, 0)
    }
}
