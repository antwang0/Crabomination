//! Odyssey (ODY) gap-closing wave 10: the pay-in-damage rares, the library
//! manipulation and the graveyard-cost engines. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    TriggeredAbility, WardCost,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, StaticEffect, Value,
    ZoneDest,
    shortcut::{draw, target_filtered},
};
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

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

/// An Aura enchanting whatever `enchant` matches.
fn aura(name: &'static str, c: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..enchantment(name, c)
    }
}

/// "…unless that player has this deal `n` damage to them" — the Odyssey
/// pay-in-damage menu. `then` is what happens when they decline.
fn unless_takes(who: PlayerRef, n: u32, then: Effect) -> Effect {
    Effect::UnlessPlayerPays {
        who,
        cost: WardCost::DamageFromSource(n),
        then: Box::new(then),
        if_paid: None,
    }
}

// ── Pay in damage ───────────────────────────────────────────────────────────

/// Blazing Salvo — {R}. 3 to a creature, or 5 to its controller's face.
pub fn blazing_salvo() -> CardDefinition {
    instant(
        "Blazing Salvo",
        cost(&[r()]),
        unless_takes(
            PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            5,
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
        ),
    )
}

/// Lava Blister — {1}{R}. Kill a nonbasic land, or take 6 for it.
pub fn lava_blister() -> CardDefinition {
    sorcery(
        "Lava Blister",
        cost(&[generic(1), r()]),
        unless_takes(
            PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            6,
            Effect::Destroy { what: target_filtered(R::Land.and(R::IsNonbasicLand)) },
        ),
    )
}

/// Molten Influence — {1}{R}. Counter a spell, or its controller takes 4.
pub fn molten_influence() -> CardDefinition {
    instant(
        "Molten Influence",
        cost(&[generic(1), r()]),
        unless_takes(
            PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            4,
            Effect::CounterSpell {
                what: target_filtered(
                    R::IsSpellOnStack
                        .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
            },
        ),
    )
}

// ── Library manipulation ────────────────────────────────────────────────────

/// Bamboozle — {2}{U}. Reveal four, bin two.
pub fn bamboozle() -> CardDefinition {
    sorcery(
        "Bamboozle",
        cost(&[generic(2), u()]),
        Effect::RevealTopChooseToGraveyard {
            who: PlayerRef::Target(0),
            reveal: Value::Const(4),
            pick: Value::Const(2),
        },
    )
}

/// Balshan Beguiler — {2}{U} 1/1 that bins a card off every connection.
pub fn balshan_beguiler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::RevealTopChooseToGraveyard {
                who: PlayerRef::Target(0),
                reveal: Value::Const(2),
                pick: Value::ONE,
            },
        }],
        ..creature(
            "Balshan Beguiler",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Predict — {1}{U}. Name a card; guessing right doubles the draw.
pub fn predict() -> CardDefinition {
    instant(
        "Predict",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::NameCard { what: Selector::This, restrict_to: None },
            Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CardsMilledThisEffectMatching { filter: R::NamedBySource },
                    Value::ONE,
                ),
                then: Box::new(draw(2)),
                else_: Box::new(draw(1)),
            },
        ]),
    )
}

/// Charmed Pendant — {4} artifact. Mills for its own coloured pips.
pub fn charmed_pendant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::MillAddManaForColoredSymbols { who: PlayerRef::You },
            ..Default::default()
        }],
        ..artifact("Charmed Pendant", cost(&[generic(4)]))
    }
}

// ── Blue control ────────────────────────────────────────────────────────────

/// Cephalid Shrine — {1}{U}{U}. Every spell is taxed by its own copies in
/// the graveyards.
pub fn cephalid_shrine() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::CounterUnlessPaid {
                what: Selector::TriggerSource,
                mana_cost: ManaCost::default(),
                exile: false,
                extra_generic: Some(Value::CardsNamedLikeTriggerSpellInAllGraveyards),
            },
        }],
        ..enchantment("Cephalid Shrine", cost(&[generic(1), u(), u()]))
    }
}

/// Aura Graft — {1}{U}. Steal an Aura and hang it somewhere else.
pub fn aura_graft() -> CardDefinition {
    instant(
        "Aura Graft",
        cost(&[generic(1), u()]),
        Effect::GainControlAndReattachAura {
            what: target_filtered(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
        },
    )
}

/// Cultural Exchange — {4}{U}{U}. Swap creatures between two players.
pub fn cultural_exchange() -> CardDefinition {
    sorcery(
        "Cultural Exchange",
        cost(&[generic(4), u(), u()]),
        Effect::ExchangeControlChoosing {
            filter: R::Creature.and(R::ControlledByOpponent),
            with: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
        },
    )
}

/// Immobilizing Ink — {1}{U} Aura. Locks the creature down until its
/// controller pays a card.
pub fn immobilizing_ink() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                discard_cost: Some((R::Any, 1)),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Immobilizing Ink", cost(&[generic(1), u()]), R::Creature)
    }
}

/// Chamber of Manipulation — {2}{U}{U} Aura on a land that rents out
/// creatures for a card.
pub fn chamber_of_manipulation() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::GainControl {
                    what: target_filtered(R::Creature),
                    to: None,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Chamber of Manipulation", cost(&[generic(2), u(), u()]), R::Land)
    }
}

// ── White / green statics ───────────────────────────────────────────────────

/// Earnest Fellowship — {1}{W}. Nobody's removal lines up any more.
pub fn earnest_fellowship() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature has protection from its colors.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::ProtectionFromOwnColors,
            },
        }],
        ..enchantment("Earnest Fellowship", cost(&[generic(1), w()]))
    }
}

/// Holistic Wisdom — {1}{G}{G}. Trade a card in hand for a same-type card in
/// your graveyard.
pub fn holistic_wisdom() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            exile_from_hand_cost: Some(R::Any),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::SharesCardTypeWithExiledBySource,
                },
                then: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..enchantment("Holistic Wisdom", cost(&[generic(1), g(), g()]))
    }
}

/// Graceful Antelope — {2}{W}{W} 1/4 that walks over Plains and makes more
/// of them.
pub fn graceful_antelope() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Plains)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            // The printed clause is "until this creature leaves the
            // battlefield"; the engine has no such duration, so the change is
            // permanent.
            effect: Effect::BecomeBasicLand {
                what: target_filtered(R::Land),
                land_type: LandType::Plains,
                duration: Duration::Permanent,
            },
        }],
        ..creature("Graceful Antelope", cost(&[generic(2), w(), w()]), vec![CreatureType::Antelope], 1, 4)
    }
}

/// Spiritualize — {2}{W}. Turn a creature's damage into life, and cantrip.
pub fn spiritualize() -> CardDefinition {
    instant(
        "Spiritualize",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

// ── Artifacts and Threshold-adjacent bodies ─────────────────────────────────

/// Catalyst Stone — {2}. Your flashbacks get cheaper, theirs get dearer.
pub fn catalyst_stone() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Flashback costs you pay cost {2} less.",
                effect: StaticEffect::FlashbackCostReduction { amount: 2 },
            },
            StaticAbility {
                description: "Flashback costs your opponents pay cost {2} more.",
                effect: StaticEffect::OpponentFlashbackTax { amount: 2 },
            },
        ],
        ..artifact("Catalyst Stone", cost(&[generic(2)]))
    }
}

/// Savage Firecat — {3}{R}{R}. Seven counters that drain away as you tap out.
pub fn savage_firecat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(7))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
            ),
            effect: Effect::RemoveCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Savage Firecat",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Cat],
            0,
            0,
        )
    }
}

/// Pardic Firecat — {3}{R} 2/3 haste that Flame Burst counts as one of its
/// own from the graveyard.
pub fn pardic_firecat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        counts_as_named_in_graveyard: Some("Flame Burst"),
        ..creature(
            "Pardic Firecat",
            cost(&[generic(3), r()]),
            vec![CreatureType::Elemental, CreatureType::Cat],
            2,
            3,
        )
    }
}

/// Aegis of Honor — {W}. Bounce a burn spell back at its caster.
pub fn aegis_of_honor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::IsSpellOnStack
                    .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                reflect: true,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
                exile_top_per_prevented: false,
            },
            ..Default::default()
        }],
        ..enchantment("Aegis of Honor", cost(&[w()]))
    }
}

