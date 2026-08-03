//! Modern Horizons 3 (MH3), batch 3 — landfall-granted battle cry, Eldrazi
//! Spawn payoffs, modified-matters, a saga, and the "spell // land" modal DFC
//! cycle. Tests in `tests/mh3c.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    StaticAbility, Subtypes, TokenDefinition, Zone,
};
use crate::effect::shortcut::{
    adapt, battle_cry, etb, evolve, exalted, on_attack, on_cast, on_dies, target_any,
    target_filtered,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
};
use crate::mana::{Color, b, colorless, cost, g, generic, hybrid, r, u, w};

// ── Landfall-granted battle cry ──────────────────────────────────────────────

/// Reckless Pyrosurfer — {1}{R} 2/2 Human Scout with haste. Landfall: it gains
/// battle cry until end of turn (grants the `battle_cry` Attacks trigger EOT).
pub fn reckless_pyrosurfer() -> CardDefinition {
    CardDefinition {
        name: "Reckless Pyrosurfer",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Land,
                }),
            effect: Effect::GrantTriggeredAbility {
                what: Selector::This,
                trigger: Box::new(battle_cry(1)),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

// ── Eldrazi Spawn ─────────────────────────────────────────────────────────────

/// Spawn-Gang Commander — {3}{R}{R} 2/2 devoid Eldrazi Goblin. When you cast it,
/// create three Eldrazi Spawn. {1}{C}, Sacrifice an Eldrazi: 2 damage to any
/// target.
pub fn spawn_gang_commander() -> CardDefinition {
    CardDefinition {
        name: "Spawn-Gang Commander",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![on_cast(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: crabomination_base::tokens::eldrazi_spawn_token(),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), colorless(1)]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Eldrazi), 1)),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Modified-matters ──────────────────────────────────────────────────────────

/// Hydra Trainer — {1}{G} 1/1 Human Warrior. Exert as it attacks: target
/// creature gets +X/+X, where X is the number of counters on permanents you
/// control. {2}{G}: Adapt 2.
pub fn hydra_trainer() -> CardDefinition {
    CardDefinition {
        name: "Hydra Trainer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Exert],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::TotalCountersOn {
                what: Box::new(Selector::EachPermanent(R::ControlledByYou)),
            },
            toughness: Value::TotalCountersOn {
                what: Box::new(Selector::EachPermanent(R::ControlledByYou)),
            },
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: adapt(2),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Signature Slam — {2}{G} Instant. Put a +1/+1 counter on target creature you
/// control, then each modified creature you control deals damage equal to its
/// power to target creature you don't control.
pub fn signature_slam() -> CardDefinition {
    CardDefinition {
        name: "Signature Slam",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::IsModified),
                ),
                body: Box::new(Effect::DealDamageEqualToPower {
                    source: Selector::TriggerSource,
                    target: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                }),
            },
        ]),
        ..Default::default()
    }
}

// ── Artifact / recursion ──────────────────────────────────────────────────────

fn phyrexian_wurm(power: i32, toughness: i32, kw: Keyword) -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Wurm".into(),
        power,
        toughness,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wurm],
            ..Default::default()
        },
        keywords: vec![kw],
        ..Default::default()
    }
}

/// Wurmcoil Larva — {3}{B}{B} 3/3 Artifact Creature — Phyrexian Wurm with
/// deathtouch and lifelink. When it dies, create a 1/2 deathtouch token and a
/// 2/1 lifelink token (both black Phyrexian Wurm artifact creatures).
pub fn wurmcoil_larva() -> CardDefinition {
    CardDefinition {
        name: "Wurmcoil Larva",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wurm],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: phyrexian_wurm(1, 2, Keyword::Deathtouch),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: phyrexian_wurm(2, 1, Keyword::Lifelink),
            },
        ]))],
        ..Default::default()
    }
}

// ── Saga ──────────────────────────────────────────────────────────────────────

/// Cat Warrior token — 2/1 white.
fn cat_warrior() -> TokenDefinition {
    TokenDefinition {
        name: "Cat Warrior".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Ajani Fells the Godsire — {3}{W}{W} Saga. I — exile target opponent creature
/// with power 3+. II — make a 2/1 Cat Warrior and put a vigilance counter on a
/// creature you control. III — target creature you control gains double strike.
pub fn ajani_fells_the_godsire() -> CardDefinition {
    CardDefinition {
        name: "Ajani Fells the Godsire",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::Exile {
                    what: target_filtered(
                        R::Creature
                            .and(R::ControlledByOpponent)
                            .and(R::PowerAtLeast(3)),
                    ),
                },
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: cat_warrior(),
                    },
                    Effect::AddKeywordCounter {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        keyword: Keyword::Vigilance,
                        amount: Value::ONE,
                    },
                ]),
            ),
            (
                3,
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..Default::default()
    }
}

// ── Modal DFC land backs (MH3 "spell // land") ───────────────────────────────

/// Back for the MH3 mono-color modal DFCs: "As this land enters, you may pay 3
/// life. If you don't, it enters tapped. {T}: Add {color}."
fn mdfc_pain_land(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                Effect::Tap {
                    what: Selector::This,
                },
            ]),
        }],
        activated_abilities: vec![crate::catalog::sets::tap_add(color)],
        ..Default::default()
    }
}

/// Back for the MH3 dual-color modal DFCs: "This land enters tapped. {T}: Add
/// {a} or {b}."
fn mdfc_dual_tapland(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![a, b], Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Boggart Trawler // Boggart Bog — {2}{B} 3/1 Goblin. ETB: exile a graveyard.
pub fn boggart_trawler() -> CardDefinition {
    CardDefinition {
        name: "Boggart Trawler",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::EachOpponent,
                zone: Zone::Graveyard,
                filter: R::Any,
            },
            to: ZoneDest::Exile,
        })],
        back_face: Some(Box::new(mdfc_pain_land("Boggart Bog", Color::Black))),
        ..Default::default()
    }
}

/// Fell the Profane // Fell Mire — {2}{B}{B} Instant. Destroy target creature or
/// planeswalker.
pub fn fell_the_profane() -> CardDefinition {
    CardDefinition {
        name: "Fell the Profane",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.or(R::Planeswalker)),
        },
        back_face: Some(Box::new(mdfc_pain_land("Fell Mire", Color::Black))),
        ..Default::default()
    }
}

/// Razorgrass Ambush // Razorgrass Field — {1}{W} Instant. 3 damage to target
/// attacking or blocking creature.
pub fn razorgrass_ambush() -> CardDefinition {
    CardDefinition {
        name: "Razorgrass Ambush",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            amount: Value::Const(3),
        },
        back_face: Some(Box::new(mdfc_pain_land("Razorgrass Field", Color::White))),
        ..Default::default()
    }
}

/// Legion Leadership // Legion Stronghold — {1}{R/W} Instant. Double target
/// creature's power and it gains first strike until end of turn.
pub fn legion_leadership() -> CardDefinition {
    CardDefinition {
        name: "Legion Leadership",
        cost: cost(&[generic(1), hybrid(Color::Red, Color::White)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DoublePower {
                what: Selector::Target(0),
                times: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
        back_face: Some(Box::new(mdfc_dual_tapland(
            "Legion Stronghold",
            Color::Red,
            Color::White,
        ))),
        ..Default::default()
    }
}

/// Revitalizing Repast // Old-Growth Grove — {B/G} Instant. Put a +1/+1 counter
/// on target creature; it gains indestructible until end of turn.
pub fn revitalizing_repast() -> CardDefinition {
    CardDefinition {
        name: "Revitalizing Repast",
        cost: cost(&[hybrid(Color::Black, Color::Green)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        back_face: Some(Box::new(mdfc_dual_tapland(
            "Old-Growth Grove",
            Color::Black,
            Color::Green,
        ))),
        ..Default::default()
    }
}

/// Stump Stomp // Burnwillow Clearing — {1}{R/G} Sorcery. Target creature you
/// control deals damage equal to its power to target creature or planeswalker
/// you don't control.
pub fn stump_stomp() -> CardDefinition {
    CardDefinition {
        name: "Stump Stomp",
        cost: cost(&[generic(1), hybrid(Color::Red, Color::Green)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamageEqualToPower {
            source: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.and(R::ControlledByYou),
            },
            target: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
            },
        },
        back_face: Some(Box::new(mdfc_dual_tapland(
            "Burnwillow Clearing",
            Color::Red,
            Color::Green,
        ))),
        ..Default::default()
    }
}

/// Waterlogged Teachings // Inundated Archive — {3}{U/B} Instant. Search your
/// library for an instant card or a card with flash, put it into your hand.
pub fn waterlogged_teachings() -> CardDefinition {
    CardDefinition {
        name: "Waterlogged Teachings",
        cost: cost(&[generic(3), hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::HasCardType(CardType::Instant).or(R::HasKeyword(Keyword::Flash)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        back_face: Some(Box::new(mdfc_dual_tapland(
            "Inundated Archive",
            Color::Blue,
            Color::Black,
        ))),
        ..Default::default()
    }
}

// ── Aura ──────────────────────────────────────────────────────────────────────

/// Lion Umbra — {G}{G} Aura. Enchant modified creature. Enchanted creature gets
/// +3/+3 and has vigilance and reach. Umbra armor.
pub fn lion_umbra() -> CardDefinition {
    CardDefinition {
        name: "Lion Umbra",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::UmbraArmor],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.and(R::IsModified),
            },
        },
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Vigilance, Keyword::Reach],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Batch 5: more "spell // land" and "creature // land" modal DFCs + Auras ────

/// Witch Enchanter // Witch-Blessed Meadow — {3}{W} 2/2 Human Warlock. ETB:
/// destroy target artifact or enchantment an opponent controls.
pub fn witch_enchanter() -> CardDefinition {
    CardDefinition {
        name: "Witch Enchanter",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Destroy {
            what: target_filtered((R::Artifact.or(R::Enchantment)).and(R::ControlledByOpponent)),
        })],
        back_face: Some(Box::new(mdfc_pain_land(
            "Witch-Blessed Meadow",
            Color::White,
        ))),
        ..Default::default()
    }
}

/// Pinnacle Monk // Mystic Peak — {3}{R}{R} 2/2 Djinn Monk with prowess. ETB:
/// return target instant or sorcery card from your graveyard to your hand.
pub fn pinnacle_monk() -> CardDefinition {
    CardDefinition {
        name: "Pinnacle Monk",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        back_face: Some(Box::new(mdfc_pain_land("Mystic Peak", Color::Red))),
        ..Default::default()
    }
}

/// Bridgeworks Battle // Tanglespan Bridgeworks — {2}{G} Sorcery. Target creature
/// you control gets +2/+2, then fights up to one target creature you don't
/// control.
pub fn bridgeworks_battle() -> CardDefinition {
    CardDefinition {
        name: "Bridgeworks Battle",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::Fight {
                attacker: Selector::Target(0),
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
        ]),
        back_face: Some(Box::new(mdfc_pain_land(
            "Tanglespan Bridgeworks",
            Color::Green,
        ))),
        ..Default::default()
    }
}

/// Disciple of Freyalise // Garden of Freyalise — {3}{G}{G}{G} 3/3 Elf Druid.
/// ETB: you may sacrifice another creature; if you do, gain X life and draw X
/// cards, where X is that creature's power.
pub fn disciple_of_freyalise() -> CardDefinition {
    CardDefinition {
        name: "Disciple of Freyalise",
        cost: cost(&[generic(3), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::MaySacrifice {
            description: "Sacrifice another creature: gain and draw = its power".into(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::SacrificedPower,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::SacrificedPower,
                },
            ])),
            else_: None,
        })],
        back_face: Some(Box::new(mdfc_pain_land(
            "Garden of Freyalise",
            Color::Green,
        ))),
        ..Default::default()
    }
}

/// Glasswing Grace // Age-Graced Chapel — {3}{W/B}{W/B} Aura. Enchanted creature
/// gets +2/+2 and has flying and lifelink.
pub fn glasswing_grace() -> CardDefinition {
    CardDefinition {
        name: "Glasswing Grace",
        cost: cost(&[
            generic(3),
            hybrid(Color::White, Color::Black),
            hybrid(Color::White, Color::Black),
        ]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying, Keyword::Lifelink],
            ..Default::default()
        }),
        back_face: Some(Box::new(mdfc_dual_tapland(
            "Age-Graced Chapel",
            Color::White,
            Color::Black,
        ))),
        ..Default::default()
    }
}

/// Strength of the Harvest // Haven of the Harvest — {2}{G/W} Aura. Enchanted
/// creature gets +1/+1 for each creature and/or enchantment you control.
pub fn strength_of_the_harvest() -> CardDefinition {
    use crate::card::EquipScale;
    CardDefinition {
        name: "Strength of the Harvest",
        cost: cost(&[generic(2), hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Creature.or(R::Enchantment),
                per_power: 1,
                per_toughness: 1,
                count_self_counters: None,
                count_graveyard: None,
                count_all_graveyards: None,
                count_host_colors: false,
                ..Default::default()
            }),
            ..Default::default()
        }),
        back_face: Some(Box::new(mdfc_dual_tapland(
            "Haven of the Harvest",
            Color::Green,
            Color::White,
        ))),
        ..Default::default()
    }
}

// ── Static P/T + type lords ──────────────────────────────────────────────────

/// Kudo, King Among Bears — {G}{W} 2/2 legendary Bear. Other creatures have
/// base power and toughness 2/2 and are Bears in addition to their other types.
pub fn kudo_king_among_bears() -> CardDefinition {
    let others = || Selector::EachPermanent(R::Creature.and(R::OtherThanSource));
    CardDefinition {
        name: "Kudo, King Among Bears",
        cost: cost(&[g(), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![
            StaticAbility {
                description: "Other creatures have base power and toughness 2/2.",
                effect: StaticEffect::SetBasePtForFilter {
                    applies_to: others(),
                    power: 2,
                    toughness: 2,
                },
            },
            StaticAbility {
                description: "Other creatures are Bears in addition to their other types.",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: others(),
                    creature_type: CreatureType::Bear,
                },
            },
        ],
        ..Default::default()
    }
}

// ── Colorless-spent payoff ({C} matters) ─────────────────────────────────────

/// Drowner of Truth — {5}{G/U}{G/U} 7/6 devoid Eldrazi. When you cast this
/// spell, if {C} was spent to cast it, create two Eldrazi Spawn.
pub fn drowner_of_truth() -> CardDefinition {
    CardDefinition {
        name: "Drowner of Truth",
        cost: cost(&[
            generic(5),
            hybrid(Color::Green, Color::Blue),
            hybrid(Color::Green, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![on_cast(Effect::If {
            cond: Predicate::CastSpellColorlessManaSpent { spent: true },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: crabomination_base::tokens::eldrazi_spawn_token(),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Emissary of Soulfire — {1}{W}{U} 1/4 Djinn Monk. ETB: get {E}{E}{E}. {E}{E}:
/// put an exalted counter on target creature you control (sorcery speed). Each
/// exalted counter is modeled as a permanently-granted `exalted()` instance.
pub fn emissary_of_soulfire() -> CardDefinition {
    CardDefinition {
        name: "Emissary of Soulfire",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::AddEnergy(
            Value::Const(3),
        ))],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 2,
            sorcery_speed: true,
            effect: Effect::GrantTriggeredAbility {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                trigger: Box::new(exalted()),
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Inventor's Axe — {R} Equipment with flash. ETB: get {E}{E} and attach to a
/// creature you control. Equipped creature gets +2/+0. Equip—Pay {E}{E}.
pub fn inventors_axe() -> CardDefinition {
    CardDefinition {
        name: "Inventor's Axe",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[]))],
        equip_energy_cost: 2,
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 0,
            ..Default::default()
        }),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::AddEnergy(Value::Const(2))),
            crate::effect::shortcut::etb(Effect::Attach {
                what: Selector::This,
                to: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
            }),
        ],
        ..Default::default()
    }
}

/// Monumental Henge — a Land that enters tapped unless you control a Plains.
/// {T}: Add {W}. {2}{W}{W}, {T}: look at the top five, put a historic card into
/// your hand, the rest on the bottom.
pub fn monumental_henge() -> CardDefinition {
    use crate::card::{LandType, Supertype};
    let historic = || {
        R::Artifact
            .or(R::HasSupertype(Supertype::Legendary))
            .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga))
    };
    CardDefinition {
        name: "Monumental Henge",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasLandType(LandType::Plains).and(R::ControlledByYou),
                )),
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Tap {
                    what: Selector::This,
                }),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::White]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w(), w()]),
                tap_cost: true,
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(5),
                    rest_to_graveyard: false,
                    pick_filter: Some(historic()),
                    take: None,
                    to_battlefield: false,
                    gain_life_if_pick: None,
                    gain_life_greatest_power_rest: false,
                    optional: false,
                    picked_lands_to_battlefield: false,
                    rest_bottom_random: false,
                    rest_to_exile: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Spymaster's Vault — a Land that enters tapped unless you control a Swamp.
/// {T}: Add {B}. {B}, {T}: target creature you control connives X, where X is
/// the number of creatures that died this turn.
pub fn spymasters_vault() -> CardDefinition {
    use crate::card::{CounterType, LandType};
    let x = || Value::CreaturesDiedThisTurnTotal;
    CardDefinition {
        name: "Spymaster's Vault",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasLandType(LandType::Swamp).and(R::ControlledByYou),
                )),
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Tap {
                    what: Selector::This,
                }),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Black]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: x(),
                    },
                    Effect::Discard {
                        who: Selector::You,
                        amount: x(),
                        random: false,
                    },
                    Effect::AddCounter {
                        what: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::Creature.and(R::ControlledByYou),
                        },
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::count(Selector::DiscardedThisResolution {
                            filter: R::Nonland,
                        }),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Idol of False Gods — {2} Kindred Artifact — Eldrazi. {1}{C}, {T}: make an
/// Eldrazi Spawn. Another Eldrazi you control dying grows it; with eight or
/// more +1/+1 counters it's a 0/0 creature with annihilator 2.
pub fn idol_of_false_gods() -> CardDefinition {
    CardDefinition {
        name: "Idol of False Gods",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Kindred, CardType::Artifact],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), colorless(1)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::eldrazi_spawn_token(),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Eldrazi),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "With eight or more +1/+1 counters, it's a 0/0 creature.",
                effect: StaticEffect::SelfIsCreatureWhileCountersAtLeast {
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    n: 8,
                },
            },
            StaticAbility {
                description: "With eight or more +1/+1 counters, it has annihilator 2.",
                effect: StaticEffect::SelfHasKeywordWhileCountersAtLeast {
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    n: 8,
                    keyword: Keyword::Annihilator(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Aether Revolt — {2}{R}{R} Enchantment. Revolt: while a permanent left the
/// battlefield under your control this turn, your noncombat damage to opponents
/// (and their permanents) is dealt +2. Whenever you get one or more {E}, deal
/// that much damage to any target.
pub fn aether_revolt() -> CardDefinition {
    CardDefinition {
        name: "Aether Revolt",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Revolt — your noncombat damage to opponents is dealt plus 2.",
            effect: StaticEffect::NoncombatDamageToOpponentsBonus {
                amount: 2,
                while_revolt: true,
            },
        }],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::EnergyGained, EventScope::YourControl),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

/// Monstrous Vortex — {3}{G} Enchantment. Whenever you cast a creature spell
/// with power 5 or greater, discover X, where X is that spell's mana value.
pub fn monstrous_vortex() -> CardDefinition {
    CardDefinition {
        name: "Monstrous Vortex",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtLeast(5)),
                },
            ),
            effect: Effect::Discover {
                n: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                filter: None,
            },
        }],
        ..Default::default()
    }
}

/// Bespoke Battlewagon — {3}{U} 5/6 Vehicle, Crew 4. Energy engine: tap for
/// {E}{E}; spend energy to tap a creature, draw, or self-animate for a turn.
pub fn bespoke_battlewagon() -> CardDefinition {
    CardDefinition {
        name: "Bespoke Battlewagon",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Crew(4)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddEnergy(Value::Const(2)),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                energy_cost: 2,
                effect: Effect::Tap {
                    what: target_filtered(R::Creature),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                energy_cost: 3,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
            ActivatedAbility {
                energy_cost: 4,
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(5),
                    toughness: Value::Const(6),
                    creature_types: vec![],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Imskir Iron-Eater — {6}{B}{R} 5/5 legendary Demon. Affinity for artifacts;
/// ETB draw/lose X (half your artifacts, rounded down); {3}{R}, sacrifice an
/// artifact: deal damage equal to its mana value to any target.
pub fn imskir_iron_eater() -> CardDefinition {
    let half_artifacts = || {
        Value::HalvedRoundDown(Box::new(Value::CountMatching {
            sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
            filter: R::Artifact,
        }))
    };
    CardDefinition {
        name: "Imskir Iron-Eater",
        cost: cost(&[generic(6), b(), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        affinity_filter: Some(R::Artifact.and(R::ControlledByYou)),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: half_artifacts(),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: half_artifacts(),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::SacrificedManaValue,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Deem Inferior — {3}{U} Sorcery. Costs {1} less for each card you've drawn
/// this turn; the owner of target nonland permanent puts it into their library
/// second from the top or on the bottom.
pub fn deem_inferior() -> CardDefinition {
    CardDefinition {
        name: "Deem Inferior",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_per_cards_drawn: true,
        effect: Effect::Move {
            what: target_filtered(R::Nonland),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: crate::effect::LibraryPosition::SecondFromTopOrBottom,
            },
        },
        ..Default::default()
    }
}

/// Path of Annihilation — {3}{G} devoid Enchantment. ETB make two Eldrazi
/// Spawn; Eldrazi you control tap for any color; casting a MV-7+ creature
/// spell gains you 4 life.
pub fn path_of_annihilation() -> CardDefinition {
    CardDefinition {
        name: "Path of Annihilation",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: crabomination_base::tokens::eldrazi_spawn_token(),
            }),
            crate::card::TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::ManaValueAtLeast(7)),
                    },
                ),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(4),
                },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Eldrazi you control have \"{T}: Add one mana of any color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Eldrazi).and(R::ControlledByYou),
                ),
                ability: super::tap_add_any_color(),
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Propagator Drone — {1}{G} 2/2 devoid Eldrazi Drone. Creature tokens you
/// control have evolve. {3}{G}: Create a 0/1 Eldrazi Spawn.
pub fn propagator_drone() -> CardDefinition {
    CardDefinition {
        name: "Propagator Drone",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Devoid],
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control have evolve.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::ControlledByYou).and(R::IsToken),
                ability: Box::new(evolve()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::eldrazi_spawn_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wumpus Aberration — {3}{G} 6/6 devoid Eldrazi Beast with trample. When you
/// cast this spell, if {C} wasn't spent to cast it, target opponent may put a
/// creature card from their hand onto the battlefield.
pub fn wumpus_aberration() -> CardDefinition {
    CardDefinition {
        name: "Wumpus Aberration",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Devoid, Keyword::Trample],
        triggered_abilities: vec![on_cast(Effect::If {
            cond: Predicate::CastSpellColorlessManaSpent { spent: false },
            then: Box::new(Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::EachOpponent,
                filter: R::Creature,
                count: Value::Const(1),
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

// ── Untap-lock artifact ──────────────────────────────────────────────────────

/// Winter Moon — {2} artifact. Players can't untap more than one nonbasic land
/// during their untap steps.
pub fn winter_moon() -> CardDefinition {
    CardDefinition {
        name: "Winter Moon",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Players can't untap more than one nonbasic land during their untap steps.",
            effect: StaticEffect::MaxOneUntapPerStep {
                filter: R::IsNonbasicLand,
            },
        }],
        ..Default::default()
    }
}

/// Cursed Wombat — {B}{G} 2/3 Nightmare Wombat. {2}{B}{G}: Adapt 2. Permanents
/// you control have "Whenever one or more +1/+1 counters are put on this
/// permanent, put an additional +1/+1 counter on it. Only once each turn."
pub fn cursed_wombat() -> CardDefinition {
    CardDefinition {
        name: "Cursed Wombat",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Wombat],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), g()]),
            effect: adapt(2),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Permanents you control have \"Whenever one or more +1/+1 counters are put on this permanent, put an additional +1/+1 counter on it. This triggers only once each turn.\"",
            effect: StaticEffect::CounterAmplifierOncePerTurn,
        }],
        ..Default::default()
    }
}

/// Rush of Inspiration // Crackling Falls — {1}{U/R}{U/R} instant. Draw two,
/// then discard a card at random unless you pay {E}{E}. Back: U/R tapland.
pub fn rush_of_inspiration() -> CardDefinition {
    CardDefinition {
        name: "Rush of Inspiration",
        cost: cost(&[
            generic(1),
            hybrid(Color::Blue, Color::Red),
            hybrid(Color::Blue, Color::Red),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::PayEnergyOrElse {
                amount: 2,
                otherwise: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: true,
                }),
            },
        ]),
        back_face: Some(Box::new(mdfc_dual_tapland(
            "Crackling Falls",
            Color::Blue,
            Color::Red,
        ))),
        ..Default::default()
    }
}

/// Rosecot Knight — {4}{W} 3/4 Human Knight with vigilance. ETB: look at the top
/// six, put an artifact or enchantment from among them into hand, rest on the
/// bottom. (The "+1/+1 counter if you didn't take one" consolation is dropped.)
pub fn rosecot_knight() -> CardDefinition {
    CardDefinition {
        name: "Rosecot Knight",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(6),
            rest_to_graveyard: false,
            pick_filter: Some(R::Artifact.or(R::Enchantment)),
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
            rest_to_exile: false,
        })],
        ..Default::default()
    }
}
