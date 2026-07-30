//! Equipment / Voltron batch. Equipment ride the `equipped_bonus` layer path;
//! payoffs ride existing primitives (Attach, `IsEquipped`/`IsEnchanted` attack
//! filters, `AttachedToMe` counts, `Effect::Search` to battlefield/hand) plus
//! one new CDA (`DynamicPt::ArtifactsControlledPower` — Akiri). Tests in
//! `tests/recent94.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, DynamicPt, Effect,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    Selector, StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, gain_life, on_attack, target_filtered};
use crate::effect::{Duration, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, w};

use super::modern::simple_equipment;

/// A target Equipment you control.
fn equipment_you_control() -> Selector {
    target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou))
}

// ── Equipment ────────────────────────────────────────────────────────────────

/// Grafted Wargear — {3} Equipment. Equipped creature gets +3/+2. Equip {0}.
/// (The "becomes unattached → sacrifice that permanent" rider is dropped.)
pub fn grafted_wargear() -> CardDefinition {
    simple_equipment(
        "Grafted Wargear",
        cost(&[generic(3)]),
        cost(&[]),
        3,
        2,
        vec![],
    )
}

/// Bloodforged Battle-Axe — {1} Equipment. Equipped creature gets +2/+0.
/// Whenever it deals combat damage to a player, create a token copy of this
/// Equipment. Equip {2}.
pub fn bloodforged_battle_axe() -> CardDefinition {
    CardDefinition {
        name: "Bloodforged Battle-Axe",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            triggers_on_equipment: true,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CreateTokenCopyOf {
                    extra_keywords: vec![],
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Hammer of Nazahn — {4} Legendary Equipment. Whenever Hammer or another
/// Equipment you control enters, you may attach that Equipment to target
/// creature you control. Equipped creature gets +2/+0 and has indestructible.
/// Equip {4}.
pub fn hammer_of_nazahn() -> CardDefinition {
    CardDefinition {
        name: "Hammer of Nazahn",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            keywords: vec![Keyword::Indestructible],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                }),
            effect: Effect::MayDo {
                description: "Attach that Equipment to target creature you control".into(),
                body: Box::new(Effect::Attach {
                    what: Selector::TriggerSource,
                    to: target_filtered(R::Creature.and(R::ControlledByYou)),
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Voltron payoffs ──────────────────────────────────────────────────────────

/// Reyav, Master Smith — {R}{W} 2/2 Dwarf Artificer. Whenever a creature you
/// control that's enchanted or equipped attacks, it gains double strike until
/// end of turn.
pub fn reyav_master_smith() -> CardDefinition {
    CardDefinition {
        name: "Reyav, Master Smith",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::IsEnchanted.or(R::IsEquipped)),
                },
            ),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Wyleth, Soul of Steel — {2}{R}{W} 4/4 Human Warrior, trample. Whenever Wyleth
/// attacks, draw a card for each Aura and Equipment attached to it.
pub fn wyleth_soul_of_steel() -> CardDefinition {
    CardDefinition {
        name: "Wyleth, Soul of Steel",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_attack(Effect::Draw {
            who: Selector::You,
            amount: Value::CountOf(Box::new(Selector::AttachedToMe(Box::new(Selector::This)))),
        })],
        ..Default::default()
    }
}

/// Kazuul's Toll Collector — {2}{R} 3/2 Ogre Warrior. {0}: Attach target
/// Equipment you control to this creature. Activate only as a sorcery.
pub fn kazuuls_toll_collector() -> CardDefinition {
    CardDefinition {
        name: "Kazuul's Toll Collector",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            effect: Effect::Attach {
                what: equipment_you_control(),
                to: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stonehewer Giant — {3}{W}{W} 4/4 Giant Warrior, vigilance. {1}{W}, {T}:
/// Search your library for an Equipment card and put it onto the battlefield.
/// (The immediate "attach it to a creature you control" is approximated — the
/// Equipment lands unattached and can be equipped normally.)
pub fn stonehewer_giant() -> CardDefinition {
    CardDefinition {
        name: "Stonehewer Giant",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nazahn, Revered Bladesmith — {4}{G}{W} 5/4 Cat Artificer. ETB: search your
/// library for an Equipment card and put it into your hand. Whenever an equipped
/// creature you control attacks, you may tap target creature an opponent
/// controls. (The Hammer-of-Nazahn-to-battlefield special case is approximated
/// as a plain tutor to hand.)
pub fn nazahn_revered_bladesmith() -> CardDefinition {
    CardDefinition {
        name: "Nazahn, Revered Bladesmith",
        cost: cost(&[generic(4), g(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Artificer],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::IsEquipped),
                    },
                ),
                effect: Effect::MayDo {
                    description: "Tap target creature an opponent controls".into(),
                    body: Box::new(Effect::Tap {
                        what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Goreclaw, Terror of Qal Sisma — {3}{G} 4/3 Bear. Creature spells you cast
/// with power 4 or greater cost {2} less. Whenever Goreclaw attacks, each
/// creature you control with power 4 or greater gets +1/+1 and gains trample
/// until end of turn.
pub fn goreclaw_terror_of_qal_sisma() -> CardDefinition {
    let big =
        || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)));
    CardDefinition {
        name: "Goreclaw, Terror of Qal Sisma",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast with power 4 or greater cost {2} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::Creature.and(R::PowerAtLeast(4)),
                amount: 2,
            },
        }],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: big(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: big(),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Akiri, Line-Slinger — {R}{W} 0/3 Kor Soldier Ally, first strike & vigilance.
/// Akiri gets +1/+0 for each artifact you control.
pub fn akiri_line_slinger() -> CardDefinition {
    CardDefinition {
        name: "Akiri, Line-Slinger",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Soldier, CreatureType::Ally],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower {
            base_p: 0,
            base_t: 3,
        }),
        ..Default::default()
    }
}

/// Rograkh, Son of Rohgahh — {0} 0/1 Kobold Warrior with first strike, menace,
/// and trample.
pub fn rograkh_son_of_rohgahh() -> CardDefinition {
    CardDefinition {
        name: "Rograkh, Son of Rohgahh",
        cost: cost(&[generic(0)]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kobold, CreatureType::Warrior],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike, Keyword::Menace, Keyword::Trample],
        ..Default::default()
    }
}

/// Steelshaper Apprentice — {2}{W}{W} 1/3 Human Soldier. {W}, {T}, Return this
/// creature to its owner's hand: Search your library for an Equipment card and
/// put it into your hand.
pub fn steelshaper_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Steelshaper Apprentice",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            return_self_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sylvia Brightspear — {2}{W} 2/2 Human Knight, double strike. Dragons you
/// control have double strike. (Partner and the "your team" widening are
/// dropped — the static covers creatures you control.)
pub fn sylvia_brightspear() -> CardDefinition {
    CardDefinition {
        name: "Sylvia Brightspear",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "Dragons you control have double strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Dragon).and(R::ControlledByYou),
                ),
                keyword: Keyword::DoubleStrike,
            },
        }],
        ..Default::default()
    }
}

// ── More Equipment ───────────────────────────────────────────────────────────

/// Silverskin Armor — {2} Equipment. Equipped creature gets +1/+1. Equip {2}.
/// (The "is an artifact in addition to its other types" rider is dropped.)
pub fn silverskin_armor() -> CardDefinition {
    simple_equipment(
        "Silverskin Armor",
        cost(&[generic(2)]),
        cost(&[generic(2)]),
        1,
        1,
        vec![],
    )
}

/// O-Naginata — {1} Equipment. Equipped creature gets +3/+0 and has trample.
/// Equip {2}. (The "attach only to power 3+" restriction is dropped.)
pub fn o_naginata() -> CardDefinition {
    simple_equipment(
        "O-Naginata",
        cost(&[generic(1)]),
        cost(&[generic(2)]),
        3,
        0,
        vec![Keyword::Trample],
    )
}

/// Prowler's Helm — {2} Equipment. Equipped creature can't be blocked except by
/// Walls. Equip {2}.
pub fn prowlers_helm() -> CardDefinition {
    simple_equipment(
        "Prowler's Helm",
        cost(&[generic(2)]),
        cost(&[generic(2)]),
        0,
        0,
        vec![Keyword::CantBeBlockedExceptBy(Box::new(
            R::HasCreatureType(CreatureType::Wall),
        ))],
    )
}

/// Vorpal Sword — {B} Equipment. Equipped creature gets +2/+0 and has
/// deathtouch. Equip {B}{B}. (The {5}{B}{B}{B} "loses the game" grant is
/// dropped.)
pub fn vorpal_sword() -> CardDefinition {
    simple_equipment(
        "Vorpal Sword",
        cost(&[b()]),
        cost(&[b(), b()]),
        2,
        0,
        vec![Keyword::Deathtouch],
    )
}

/// Argentum Armor — {6} Equipment. Equipped creature gets +6/+6. Whenever it
/// attacks, destroy target permanent. Equip {6}.
pub fn argentum_armor() -> CardDefinition {
    CardDefinition {
        name: "Argentum Armor",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(6)]))],
        equipped_bonus: Some(EquipBonus {
            power: 6,
            toughness: 6,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Destroy {
                    what: target_filtered(R::Permanent),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── More Voltron payoffs ─────────────────────────────────────────────────────

/// Kwende, Pride of Femeref — {3}{W} 2/2 Human Knight, double strike. Creatures
/// you control with first strike have double strike.
pub fn kwende_pride_of_femeref() -> CardDefinition {
    CardDefinition {
        name: "Kwende, Pride of Femeref",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with first strike have double strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasKeyword(Keyword::FirstStrike)),
                ),
                keyword: Keyword::DoubleStrike,
            },
        }],
        ..Default::default()
    }
}

/// Kemba's Skyguard — {1}{W}{W} 2/2 Cat Knight, flying. ETB: you gain 2 life.
pub fn kembas_skyguard() -> CardDefinition {
    CardDefinition {
        name: "Kemba's Skyguard",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(gain_life(2))],
        ..Default::default()
    }
}
