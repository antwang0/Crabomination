//! Fifth Dawn (5DN) gap batch 3 — the Bringer cycle, the sunburst artifacts,
//! the alternate-attach Equipment and the utility rares. Tests in
//! `recent_b/fdn5`.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec,
    Keyword, Predicate, Selector, SelectionRequirement as R, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, ManaCost};

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

/// An Aura with `enchant` as its printed enchant filter and `bonus` as its
/// continuous grant.
fn aura(
    name: &'static str,
    mana: ManaCost,
    enchant: R,
    bonus: EquipBonus,
) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, mana)
    }
}

/// Equipment with a printed equip cost plus the 5DN "colored mana: attach it"
/// second attach ability.
fn quick_equipment(
    name: &'static str,
    mana: ManaCost,
    attach: ManaCost,
    equip: ManaCost,
    bonus: EquipBonus,
) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: attach,
            effect: Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature.and(R::ControlledByYou)),
            },
            ..Default::default()
        }],
        ..artifact(name, mana)
    }
}

/// "At the beginning of your upkeep, [effect]."
fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource)
            .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
        effect,
    }
}

/// The WUBRG alternative cost shared by the Bringer cycle (CR 118.9).
fn wubrg_alt() -> AlternativeCost {
    AlternativeCost { mana_cost: cost(&[w(), u(), b(), r(), g()]), ..Default::default() }
}

/// A Bringer: a 5/5 trampler castable for WUBRG with a "you may" upkeep payoff.
fn bringer(name: &'static str, mana: ManaCost, upkeep: Effect, prompt: &str) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bringer],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        alternative_cost: Some(wubrg_alt()),
        triggered_abilities: vec![your_upkeep(Effect::MayDo {
            description: prompt.into(),
            body: Box::new(upkeep),
        })],
        ..Default::default()
    }
}

// ── The Bringer cycle ──

/// Bringer of the White Dawn — an artifact out of your graveyard each upkeep.
pub fn bringer_of_the_white_dawn() -> CardDefinition {
    bringer(
        "Bringer of the White Dawn",
        cost(&[generic(7), w(), w()]),
        Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::InYourGraveyard.and(R::Artifact),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        "Return target artifact card from your graveyard to the battlefield?",
    )
}

/// Bringer of the Blue Dawn — two cards each upkeep.
pub fn bringer_of_the_blue_dawn() -> CardDefinition {
    bringer(
        "Bringer of the Blue Dawn",
        cost(&[generic(7), u(), u()]),
        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        "Draw two cards?",
    )
}

/// Bringer of the Black Dawn — pay 2 life to set up your next draw.
pub fn bringer_of_the_black_dawn() -> CardDefinition {
    bringer(
        "Bringer of the Black Dawn",
        cost(&[generic(7), b(), b()]),
        Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Any,
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            },
        ]),
        "Pay 2 life to tutor a card to the top?",
    )
}

/// Bringer of the Red Dawn — borrow a creature with haste each upkeep.
pub fn bringer_of_the_red_dawn() -> CardDefinition {
    bringer(
        "Bringer of the Red Dawn",
        cost(&[generic(7), r(), r()]),
        Effect::Seq(vec![
            Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            Effect::GainControl {
                what: Selector::Target(0),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        "Untap and steal target creature until end of turn?",
    )
}

/// Bringer of the Green Dawn — a 3/3 Beast each upkeep.
pub fn bringer_of_the_green_dawn() -> CardDefinition {
    bringer(
        "Bringer of the Green Dawn",
        cost(&[generic(7), g(), g()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Beast".into(),
                colors: vec![crate::mana::Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Beast],
                    ..Default::default()
                },
                power: 3,
                toughness: 3,
                ..Default::default()
            },
        },
        "Create a 3/3 Beast?",
    )
}

// ── Mana and cost artifacts ──

/// Fist of Suns — every spell you cast may be paid for with WUBRG.
pub fn fist_of_suns() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may pay {W}{U}{B}{R}{G} rather than a spell's mana cost",
            effect: StaticEffect::FiveColorAlternativeCost,
        }],
        ..artifact("Fist of Suns", cost(&[generic(3)]))
    }
}

/// Doubling Cube — turns a floating pool into twice the pool.
pub fn doubling_cube() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::DoubleUnspentMana,
            ..Default::default()
        }],
        ..artifact("Doubling Cube", cost(&[generic(2)]))
    }
}

/// Vedalken Orrery — everything you cast has flash.
pub fn vedalken_orrery() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may cast spells as though they had flash",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: R::Any },
        }],
        ..artifact("Vedalken Orrery", cost(&[generic(4)]))
    }
}

/// Vedalken Shackles — holds a creature for as long as it stays tapped.
pub fn vedalken_shackles() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::GainControlWhileSourceTapped {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::PowerAtMostYourCount(Box::new(
                        R::HasLandType(crate::card::LandType::Island).and(R::ControlledByYou),
                    ))),
                },
            },
            ..Default::default()
        }],
        ..artifact("Vedalken Shackles", cost(&[generic(3)]))
    }
}

/// Door to Nothingness — ten pips of the right colors ends a player.
pub fn door_to_nothingness() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enters tapped",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w(), u(), u(), b(), b(), r(), r(), g(), g()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::LoseGame { who: PlayerRef::Target(0) },
            ..Default::default()
        }],
        ..artifact("Door to Nothingness", cost(&[generic(5)]))
    }
}

// ── Sunburst artifacts ──

/// Clearwater Goblet — pays back its sunburst counters in life every upkeep.
pub fn clearwater_goblet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sunburst],
        triggered_abilities: vec![your_upkeep(Effect::MayDo {
            description: "Gain life equal to the charge counters?".into(),
            body: Box::new(Effect::GainLife {
                who: Selector::You,
                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
            }),
        })],
        ..artifact("Clearwater Goblet", cost(&[generic(5)]))
    }
}

/// Heliophial — cashes its sunburst counters in as one burn spell.
pub fn heliophial() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sunburst],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
            },
            ..Default::default()
        }],
        ..artifact("Heliophial", cost(&[generic(5)]))
    }
}

/// Infused Arrows — spends charge counters shrinking a creature.
pub fn infused_arrows() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sunburst],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_x: Some(CounterType::Charge),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
                toughness: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Infused Arrows", cost(&[generic(4)]))
    }
}

/// Opaline Bracers — a sunburst Equipment that pumps by its own counters.
pub fn opaline_bracers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sunburst, Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                per_power: 1,
                per_toughness: 1,
                count_self_counters: Some(CounterType::Charge),
                ..Default::default()
            }),
            ..Default::default()
        }),
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        ..artifact("Opaline Bracers", cost(&[generic(4)]))
    }
}

// ── Equipment with a colored attach ability ──

/// Sparring Collar — first strike, attachable for {R}{R} at instant speed.
pub fn sparring_collar() -> CardDefinition {
    quick_equipment(
        "Sparring Collar",
        cost(&[generic(2)]),
        cost(&[r(), r()]),
        cost(&[generic(1)]),
        EquipBonus { keywords: vec![Keyword::FirstStrike], ..Default::default() },
    )
}

/// Horned Helm — +1/+1 and trample, attachable for {G}{G}.
pub fn horned_helm() -> CardDefinition {
    quick_equipment(
        "Horned Helm",
        cost(&[generic(2)]),
        cost(&[g(), g()]),
        cost(&[generic(1)]),
        EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Trample], ..Default::default() },
    )
}

/// Neurok Stealthsuit — shroud, attachable for {U}{U} in response to removal.
pub fn neurok_stealthsuit() -> CardDefinition {
    quick_equipment(
        "Neurok Stealthsuit",
        cost(&[generic(2)]),
        cost(&[u(), u()]),
        cost(&[generic(1)]),
        EquipBonus { keywords: vec![Keyword::Shroud], ..Default::default() },
    )
}

/// Healer's Headdress — +0/+2 and a granted damage-prevention tap ability.
pub fn healers_headdress() -> CardDefinition {
    quick_equipment(
        "Healer's Headdress",
        cost(&[generic(2)]),
        cost(&[w(), w()]),
        cost(&[generic(1)]),
        EquipBonus {
            toughness: 2,
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::PreventNextDamage {
                    target: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                    amount: Value::ONE,
                },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Ensouled Scimitar — a +1/+5 Equipment that can animate itself instead.
pub fn ensouled_scimitar() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 5, ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(5),
                creature_types: vec![CreatureType::Spirit],
                keywords: vec![Keyword::Flying],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Ensouled Scimitar", cost(&[generic(3)]))
    }
}

// ── Utility artifacts ──

/// Goblin Cannon — one shot, then it's gone.
pub fn goblin_cannon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                    amount: Value::ONE,
                },
                Effect::SacrificeSource,
            ]),
            ..Default::default()
        }],
        ..artifact("Goblin Cannon", cost(&[generic(4)]))
    }
}

/// Salvaging Station — untaps on every death to rebuy cheap artifacts.
pub fn salvaging_station() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::InYourGraveyard
                        .and(R::Artifact)
                        .and(R::Noncreature)
                        .and(R::ManaValueAtMost(1)),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::MayDo {
                description: "Untap Salvaging Station?".into(),
                body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
            },
        }],
        ..artifact("Salvaging Station", cost(&[generic(6)]))
    }
}

/// Summoning Station — a Pincher a turn, more when artifacts die.
pub fn summoning_station() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Pincher".into(),
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Pincher],
                        ..Default::default()
                    },
                    power: 2,
                    toughness: 2,
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
            ),
            effect: Effect::MayDo {
                description: "Untap Summoning Station?".into(),
                body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
            },
        }],
        ..artifact("Summoning Station", cost(&[generic(7)]))
    }
}

/// Staff of Domination — five abilities and a {1} untap to reuse them.
pub fn staff_of_domination() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::Tap { what: target_filtered(R::Creature) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..artifact("Staff of Domination", cost(&[generic(3)]))
    }
}

/// Avarice Totem — trades itself for anything on the board.
pub fn avarice_totem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::ExchangeControl {
                a: Selector::This,
                b: target_filtered(R::Permanent.and(R::Nonland)),
            },
            ..Default::default()
        }],
        ..artifact("Avarice Totem", cost(&[generic(1)]))
    }
}

/// Chimeric Coils — an X/X for a turn, at any size you can pay for.
pub fn chimeric_coils() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x(), generic(1)]),
            effect: Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::XFromCost,
                    toughness: Value::XFromCost,
                    creature_types: vec![CreatureType::Construct],
                    keywords: vec![],
                    duration: Duration::Permanent,
                },
                Effect::AtNextEndStep { body: Box::new(Effect::SacrificeSource) },
            ]),
            ..Default::default()
        }],
        ..artifact("Chimeric Coils", cost(&[generic(1)]))
    }
}

/// Skullcage — punishes an opponent for holding the wrong number of cards.
pub fn skullcage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
            effect: Effect::If {
                cond: Predicate::Any(vec![
                    Predicate::ValueEquals(
                        Value::HandSizeOf(PlayerRef::ActivePlayer),
                        Value::Const(3),
                    ),
                    Predicate::ValueEquals(
                        Value::HandSizeOf(PlayerRef::ActivePlayer),
                        Value::Const(4),
                    ),
                ]),
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(2),
                }),
            },
        }],
        ..artifact("Skullcage", cost(&[generic(4)]))
    }
}

// ── Enchantments ──

/// Artificer's Intuition — trades a fat artifact for a cheap one.
pub fn artificers_intuition() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            discard_cost: Some((R::Artifact, 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Artifact.and(R::ManaValueAtMost(1)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..enchantment("Artificer's Intuition", cost(&[generic(1), u()]))
    }
}

/// Rite of Passage — anything of yours that survives damage grows.
pub fn rite_of_passage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..enchantment("Rite of Passage", cost(&[generic(2), g()]))
    }
}

/// Ion Storm — turns spare counters into burn.
pub fn ion_storm() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            remove_counter_among_kinds: Some((
                vec![CounterType::PlusOnePlusOne, CounterType::Charge],
                1,
                R::Permanent,
            )),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..enchantment("Ion Storm", cost(&[generic(2), r()]))
    }
}

/// Disruption Aura — the enchanted artifact has to pay rent every upkeep.
pub fn disruption_aura() -> CardDefinition {
    aura(
        "Disruption Aura",
        cost(&[generic(2), u()]),
        R::Artifact,
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::SacrificeSourceUnlessPayManaValue,
            }],
            ..Default::default()
        },
    )
}

/// Stasis Cocoon — shuts an artifact off entirely.
pub fn stasis_cocoon() -> CardDefinition {
    aura(
        "Stasis Cocoon",
        cost(&[generic(1), w()]),
        R::Artifact,
        EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock, Keyword::CantActivateAbilities],
            ..Default::default()
        },
    )
}
