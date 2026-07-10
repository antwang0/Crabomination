//! Modern Horizons 3 (MH3), batch 4 — Devoid Eldrazi (bounce/exile riders,
//! kicker choose-one/both, emerge tap-lock), a kicker exile with a mana-value
//! lifegain rider, and an umbra-armor Aura. Tests in `tests/mh3d.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, Predicate, Prototype,
    SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{emerge, etb, on_cast, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest, ZoneRef};
use crate::mana::{b, colorless, cost, g, generic, r, u, w, Color};

/// Ugin's Binding — {2}{U} Devoid instant. Bounce a nonland permanent you don't
/// control. From the graveyard, casting a colorless spell of mana value 7+ lets
/// you exile it to bounce every nonland permanent you don't control.
pub fn ugins_binding() -> CardDefinition {
    CardDefinition {
        name: "Ugin's Binding",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Move {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Colorless.and(R::ManaValueAtLeast(7)),
                },
            ),
            effect: Effect::MayDo {
                description: "Exile Ugin's Binding from your graveyard to bounce all?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: Selector::This },
                    Effect::Move {
                        what: Selector::EachMatching {
                            zone: ZoneRef::Battlefield,
                            filter: R::Permanent.and(R::Nonland).and(R::ControlledByOpponent),
                        },
                        to: ZoneDest::Hand(PlayerRef::EachOpponent),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Abstruse Appropriation — {2}{W}{B} Devoid instant. Exile a nonland
/// permanent; you may cast it for as long as it stays exiled, spending any
/// mana for its cost (the "colorless as any color" rider is generalized).
pub fn abstruse_appropriation() -> CardDefinition {
    CardDefinition {
        name: "Abstruse Appropriation",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Permanent.and(R::Nonland)) },
            Effect::GrantMayPlay {
                what: Selector::Target(0),
                duration: MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
        ]),
        ..Default::default()
    }
}

/// Expel the Unworthy — {1}{W} sorcery, Kicker {2}{W}. Exile a creature of mana
/// value 3 or less (any creature if kicked); its controller gains life equal to
/// its mana value.
pub fn expel_the_unworthy() -> CardDefinition {
    CardDefinition {
        name: "Expel the Unworthy",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(2), w()]))],
        // Gain life first (while the target still has a controller), then exile.
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Exile { what: target_filtered(R::Creature) }),
                else_: Box::new(Effect::Exile {
                    what: target_filtered(R::Creature.and(R::ManaValueAtMost(3))),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Twisted Riddlekeeper — {8} 5/5 Eldrazi Sphinx with flying and Emerge
/// {5}{C}{U}. When cast, tap up to two target permanents and stun each.
pub fn twisted_riddlekeeper() -> CardDefinition {
    CardDefinition {
        name: "Twisted Riddlekeeper",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Sphinx],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(emerge(cost(&[generic(5), colorless(1), u()]))),
        triggered_abilities: vec![on_cast(Effect::ApplyToTargets {
            max_targets: 2,
            filter: R::Permanent,
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Depth Defiler — {3}{U}{U} 3/5 Devoid Eldrazi, Kicker {C}. When cast, choose
/// one — bounce a creature, or you draw two then discard a card. If
/// kicked, choose both.
pub fn depth_defiler() -> CardDefinition {
    let bounce = Effect::Move {
        what: target_filtered(R::Creature),
        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
    };
    let draw_discard = Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
    ]);
    CardDefinition {
        name: "Depth Defiler",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Devoid, Keyword::Kicker(cost(&[colorless(1)]))],
        triggered_abilities: vec![on_cast(Effect::If {
            cond: Predicate::CastSpellWasKicked,
            then: Box::new(Effect::Seq(vec![bounce.clone(), draw_discard.clone()])),
            else_: Box::new(Effect::ChooseMode(vec![bounce, draw_discard])),
        })],
        ..Default::default()
    }
}

/// Dog Umbra — {1}{W} flash Aura. Enchant creature with umbra armor. (The
/// "can't attack/block while another player controls it" rider is dropped.)
pub fn dog_umbra() -> CardDefinition {
    CardDefinition {
        name: "Dog Umbra",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::UmbraArmor],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
        },
        ..Default::default()
    }
}

/// Thief of Existence — {1}{C}{G} 3/4 Devoid Eldrazi. ETB: exile up to one
/// noncreature, nonland permanent an opponent controls with mana value 4 or
/// less; if you do, it gains "When this leaves the battlefield, draw a card."
pub fn thief_of_existence() -> CardDefinition {
    CardDefinition {
        name: "Thief of Existence",
        cost: cost(&[generic(1), colorless(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: R::Permanent
                .and(R::Noncreature)
                .and(R::Nonland)
                .and(R::ControlledByOpponent)
                .and(R::ManaValueAtMost(4)),
            effect: Box::new(Effect::Seq(vec![
                Effect::Exile { what: Selector::Target(0) },
                Effect::GrantTriggeredAbility {
                    what: Selector::This,
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::PermanentLeavesBattlefield,
                            EventScope::SelfSource,
                        ),
                        effect: Effect::Draw {
                            who: Selector::Player(PlayerRef::You),
                            amount: Value::Const(1),
                        },
                    }),
                    duration: Duration::Permanent,
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Depth Charge Colossus — {9} 9/9 Dreadnought artifact creature, Prototype
/// {4}{U}{U} — 6/6. Doesn't untap during your untap step; {3}: untap it.
pub fn depth_charge_colossus() -> CardDefinition {
    CardDefinition {
        name: "Depth Charge Colossus",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dreadnought],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        prototype: Some(Box::new(Prototype { cost: cost(&[generic(4), u(), u()]), power: 6, toughness: 6 })),
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Amphibian Downpour — {2}{U} flash Aura with Storm. Enchanted creature loses
/// all abilities and is a blue Frog with base power and toughness 1/1.
pub fn amphibian_downpour() -> CardDefinition {
    CardDefinition {
        name: "Amphibian Downpour",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Storm],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
        },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((1, 1)),
            set_creature_types: Some(vec![CreatureType::Frog]),
            set_colors: Some(vec![Color::Blue]),
            remove_abilities: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Herigast, Erupting Nullkite — {9} 6/6 Eldrazi Dragon with flying and Emerge
/// {6}{R}{R}. When cast, you may exile your hand; if you do, draw three cards.
/// (The "each creature spell you cast has emerge" static is omitted.)
pub fn herigast_erupting_nullkite() -> CardDefinition {
    CardDefinition {
        name: "Herigast, Erupting Nullkite",
        cost: cost(&[generic(9)]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Dragon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(emerge(cost(&[generic(6), r(), r()]))),
        triggered_abilities: vec![on_cast(Effect::MayDo {
            description: "Exile your hand to draw three cards?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Exile {
                    what: Selector::EachMatching {
                        zone: ZoneRef::Hand(PlayerRef::You),
                        filter: R::Any,
                    },
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            ])),
        })],
        ..Default::default()
    }
}
