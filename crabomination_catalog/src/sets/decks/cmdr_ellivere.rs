//! Commander: the cards the **Virtue and Valor** precon (WOC, Ellivere of the
//! Wild Court) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_ellivere.rs`.
//!
//! Residuals (each also on its card):
//! - **Indomitable Might** — the damage always goes as though unblocked (the
//!   controller doesn't choose).
//! - **Mantle of the Ancients**, **Unfinished Business** — the returned Aura
//!   and Equipment cards are picked (greatest mana value first), not
//!   targeted.
//! - **Retether**, **Knickknack Ouphe**, **Songbirds' Blessing**,
//!   **Liberated Livestock** — each Aura's host is the engine's pick (your
//!   greatest-power legal permanent first).

use super::woe_roles::{monster_role, royal_role, sorcerer_role, virtuous_role};
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, ConditionalEquipBonus, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, w, x};
use crate::sets::tap_add;
use std::sync::Arc;

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

fn token(name: &str, color: Color, ct: CreatureType, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

/// An Aura: "Enchant [filter]" with `bonus` on the enchanted permanent.
fn aura(name: &'static str, mana: ManaCost, filter: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter } },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn aura_card() -> R {
    R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
}

fn aura_or_equipment() -> R {
    aura_card().or(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment))
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

/// Ellivere of the Wild Court — entering or attacking, a Virtuous Role on
/// another creature you control; your enchanted creatures draw on combat
/// damage to a player.
pub fn ellivere_of_the_wild_court() -> CardDefinition {
    let role = || Effect::CreateTokenAttachedTo {
        target: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
        definition: Arc::new(virtuous_role()),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(role()),
            on_attack(role()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::IsEnchanted) },
                ),
                effect: draw(Value::ONE),
            },
        ],
        ..creature(
            "Ellivere of the Wild Court",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Aura Gnarlid — smaller creatures can't block it; +1/+1 per Aura on the
/// battlefield.
pub fn aura_gnarlid() -> CardDefinition {
    let auras = Value::CountOf(Box::new(Selector::EachPermanent(aura_card())));
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerLess],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each Aura on the battlefield.",
            effect: StaticEffect::PumpPTByValue { applies_to: Selector::This, power: auras.clone(), toughness: auras },
        }],
        ..creature("Aura Gnarlid", cost(&[generic(2), g()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Bear Umbra — +2/+2 and "attacking, untap your lands"; umbra armor.
pub fn bear_umbra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::UmbraArmor],
        ..aura(
            "Bear Umbra",
            cost(&[generic(2), g(), g()]),
            R::Creature,
            EquipBonus {
                power: 2,
                toughness: 2,
                triggered_abilities: vec![on_attack(Effect::Untap { what: yours(R::Land), up_to: None })],
                ..Default::default()
            },
        )
    }
}

/// Careful Cultivation — on a creature, +1/+3, reach and "{T}: Add {G}{G}";
/// channel {1}{G}: a 1/1 Monk with "{T}: Add {G}".
pub fn careful_cultivation() -> CardDefinition {
    let mut monk = token("Human Monk", Color::Green, CreatureType::Monk, 1, 1, vec![]);
    monk.subtypes.creature_types.insert(0, CreatureType::Human);
    monk.activated_abilities = vec![tap_add(Color::Green)];
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(monk) },
            ..Default::default()
        }],
        ..aura(
            "Careful Cultivation",
            cost(&[generic(2), g()]),
            R::Artifact.or(R::Creature),
            EquipBonus {
                conditional: vec![ConditionalEquipBonus {
                    host_filter: R::Creature,
                    power: 1,
                    toughness: 3,
                    keywords: vec![Keyword::Reach],
                    condition: None,
                    set_base_pt: None,
                    activated_abilities: vec![ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::AddMana {
                            who: PlayerRef::You,
                            pool: crate::effect::ManaPayload::Colors(vec![Color::Green, Color::Green]),
                        },
                        ..Default::default()
                    }],
                }],
                ..Default::default()
            },
        )
    }
}

/// Giant Inheritance — +5/+5 and "attacking, a Monster Role on an attacking
/// creature"; back to hand when it hits the graveyard.
pub fn giant_inheritance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..aura(
            "Giant Inheritance",
            cost(&[generic(4), g()]),
            R::Creature,
            EquipBonus {
                power: 5,
                toughness: 5,
                triggered_abilities: vec![on_attack(Effect::CreateTokenAttachedTo {
                    target: target_filtered(R::Creature.and(R::IsAttacking)),
                    definition: Arc::new(monster_role()),
                })],
                ..Default::default()
            },
        )
    }
}

/// Gylwain, Casting Director — it or another nontoken creature of yours
/// entering wears a Royal, Sorcerer or Monster Role.
pub fn gylwain_casting_director() -> CardDefinition {
    let on_it = |role: TokenDefinition| Effect::CreateTokenAttachedTo {
        target: Selector::TriggerSource,
        definition: Arc::new(role),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::NotToken) },
            ),
            effect: Effect::ChooseMode(vec![
                on_it(royal_role()),
                on_it(sorcerer_role()),
                on_it(monster_role()),
            ]),
        }],
        ..creature(
            "Gylwain, Casting Director",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Human, CreatureType::Bard],
            2,
            3,
        )
    }
}

/// Indomitable Might — flash; +3/+3; it may assign its combat damage as
/// though it weren't blocked.
/// Residual: the damage always goes as though unblocked.
pub fn indomitable_might() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..aura(
            "Indomitable Might",
            cost(&[generic(3), g()]),
            R::Creature,
            EquipBonus {
                power: 3,
                toughness: 3,
                keywords: vec![Keyword::AssignsDamageAsThoughUnblocked],
                ..Default::default()
            },
        )
    }
}

/// Knickknack Ouphe — enters with X +1/+1 counters; Auras of mana value X or
/// less from the top X go onto the battlefield.
/// Residual: each Aura's host is the engine's pick.
pub fn knickknack_ouphe() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![etb(Effect::RevealTopPutAttached {
            count: Value::XFromCost,
            filter: aura_card().and(R::ManaValueAtMostXFromCost),
        })],
        ..creature("Knickknack Ouphe", cost(&[x(), g()]), vec![CreatureType::Ouphe], 1, 1)
    }
}

/// Liberated Livestock — dying, a Cat, a Bird and an Ox, each of which may
/// wear an Aura from your hand or graveyard.
/// Residual: the Aura is the engine's pick (graveyard first).
pub fn liberated_livestock() -> CardDefinition {
    let mint = |t: TokenDefinition| Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(t) };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                mint(token("Cat", Color::White, CreatureType::Cat, 1, 1, vec![Keyword::Lifelink])),
                mint(token("Bird", Color::White, CreatureType::Bird, 1, 1, vec![Keyword::Flying])),
                mint(token("Ox", Color::White, CreatureType::Ox, 2, 4, vec![])),
                Effect::ForEach {
                    selector: Selector::LastCreatedTokens,
                    body: Box::new(Effect::PutOntoBattlefieldAttached {
                        zones: vec![Zone::Graveyard, Zone::Hand],
                        filter: aura_card(),
                        host: Some(Selector::TriggerSource),
                        max: Some(Value::ONE),
                        creatures_only: true,
                    }),
                },
            ]),
        }],
        ..creature(
            "Liberated Livestock",
            cost(&[generic(5), w()]),
            vec![CreatureType::Cat, CreatureType::Bird, CreatureType::Ox],
            4,
            6,
        )
    }
}

/// Mantle of the Ancients — returns your Aura and Equipment cards attached to
/// the creature it enchants; +1/+1 per Aura and Equipment on it.
/// Residual: the cards are picked, not targeted.
pub fn mantle_of_the_ancients() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PutOntoBattlefieldAttached {
            zones: vec![Zone::Graveyard],
            filter: aura_or_equipment(),
            host: Some(Selector::AttachedTo(Box::new(Selector::This))),
            max: None,
            creatures_only: false,
        })],
        ..aura(
            "Mantle of the Ancients",
            cost(&[generic(3), w(), w()]),
            R::Creature.and(R::ControlledByYou),
            EquipBonus {
                scale: Some(EquipScale {
                    per_power: 1,
                    per_toughness: 1,
                    count_host_attachments: Some(aura_or_equipment()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
    }
}

/// Ox Drover — vigilance; Oxen can't block it; entering or attacking, target
/// opponent gets a 2/4 Ox and you draw.
pub fn ox_drover() -> CardDefinition {
    let gift = || {
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::Target(0),
                count: Value::ONE,
                definition: Arc::new(token("Ox", Color::White, CreatureType::Ox, 2, 4, vec![])),
            },
            draw(Value::ONE),
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::CantBeBlockedByCreatureType(CreatureType::Ox)],
        triggered_abilities: vec![etb(gift()), on_attack(gift())],
        ..creature("Ox Drover", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Peasant], 4, 4)
    }
}

/// Retether — every Aura card in your graveyard returns attached to a
/// creature.
/// Residual: each Aura's host is the engine's pick.
pub fn retether() -> CardDefinition {
    CardDefinition {
        name: "Retether",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PutOntoBattlefieldAttached {
            zones: vec![Zone::Graveyard],
            filter: aura_card(),
            host: None,
            max: None,
            creatures_only: true,
        },
        ..Default::default()
    }
}

/// Sage's Reverie — entering, a card per Aura you control on a creature;
/// +1/+1 per such Aura.
pub fn sages_reverie() -> CardDefinition {
    let mine = || aura_card().and(R::AttachedToCreature);
    CardDefinition {
        triggered_abilities: vec![etb(draw(Value::CountOf(Box::new(yours(mine())))))],
        ..aura(
            "Sage's Reverie",
            cost(&[generic(3), w()]),
            R::Creature,
            EquipBonus {
                scale: Some(EquipScale { filter: mine(), per_power: 1, per_toughness: 1, ..Default::default() }),
                ..Default::default()
            },
        )
    }
}

/// Songbirds' Blessing — attacking, reveal until an Aura: onto the
/// battlefield, else to hand.
/// Residual: the Aura's host is the engine's pick.
pub fn songbirds_blessing() -> CardDefinition {
    aura(
        "Songbirds' Blessing",
        cost(&[generic(3), w()]),
        R::Creature,
        EquipBonus {
            triggered_abilities: vec![on_attack(Effect::RevealUntilPutAttachedElseHand { filter: aura_card() })],
            ..Default::default()
        },
    )
}

/// Spectral Steel — +2/+2; from the graveyard, {1}{W} and exile it: another
/// Aura or Equipment card back to hand.
pub fn spectral_steel() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(aura_or_equipment().and(R::InYourGraveyard).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..aura("Spectral Steel", cost(&[generic(1), w()]), R::Creature, EquipBonus { power: 2, toughness: 2, ..Default::default() })
    }
}

/// Timber Paladin — base 3/3 with one Aura on it, 5/5 and vigilance with two,
/// 10/10, vigilance and trample with three or more.
pub fn timber_paladin() -> CardDefinition {
    let n = || Value::AttachmentsOn { what: Box::new(Selector::This), filter: aura_card() };
    let exactly = |k: i32| {
        Predicate::All(vec![
            Predicate::ValueAtLeast(n(), Value::Const(k)),
            Predicate::ValueAtMost(n(), Value::Const(k)),
        ])
    };
    let at_least = |k: i32| Predicate::ValueAtLeast(n(), Value::Const(k));
    let grant = |k: i32, kw: Keyword, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::WhileCondition {
            condition: at_least(k),
            inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: kw }),
        },
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![
            StaticAbility {
                description: "As long as this creature is enchanted by exactly one Aura, it has base power and toughness 3/3.",
                effect: StaticEffect::SetBasePtIf { condition: exactly(1), power: 3, toughness: 3 },
            },
            StaticAbility {
                description: "As long as this creature is enchanted by exactly two Auras, it has base power and toughness 5/5.",
                effect: StaticEffect::SetBasePtIf { condition: exactly(2), power: 5, toughness: 5 },
            },
            StaticAbility {
                description: "As long as this creature is enchanted by three or more Auras, it has base power and toughness 10/10.",
                effect: StaticEffect::SetBasePtIf { condition: at_least(3), power: 10, toughness: 10 },
            },
            grant(2, Keyword::Vigilance, "Two or more Auras: vigilance."),
            grant(3, Keyword::Trample, "Three or more Auras: trample."),
        ],
        ..creature("Timber Paladin", cost(&[generic(1), g()]), vec![CreatureType::Knight], 1, 1)
    }
}

/// Umbra Mystic — Auras attached to permanents you control have umbra armor.
pub fn umbra_mystic() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Auras attached to permanents you control have umbra armor.",
            effect: StaticEffect::AurasOnYourPermanentsHaveUmbraArmor,
        }],
        ..creature("Umbra Mystic", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Unfinished Business — a creature card back from your graveyard, then up to
/// two Aura / Equipment cards attached to it.
/// Residual: the Aura and Equipment cards are picked, not targeted.
pub fn unfinished_business() -> CardDefinition {
    CardDefinition {
        name: "Unfinished Business",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::PutOntoBattlefieldAttached {
                zones: vec![Zone::Graveyard],
                filter: aura_or_equipment(),
                host: Some(Selector::Target(0)),
                max: Some(Value::Const(2)),
                creatures_only: false,
            },
        ]),
        ..Default::default()
    }
}

/// Verdant Embrace — +3/+3 and "at the beginning of each upkeep, a 1/1
/// Saproling".
pub fn verdant_embrace() -> CardDefinition {
    aura(
        "Verdant Embrace",
        cost(&[generic(3), g(), g()]),
        R::Creature,
        EquipBonus {
            power: 3,
            toughness: 3,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(token("Saproling", Color::Green, CreatureType::Saproling, 1, 1, vec![])),
                },
            }],
            ..Default::default()
        },
    )
}
