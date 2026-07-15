//! A Wilds of Eldraine (WOE) legends + spells wave: Food aristocrats, Rat
//! swarm, enchantment-matters, tap-matters (the new `YouTapped` scope),
//! restricted mana (`SpendRestriction::HighMvOrX`), and a Bargain dig. Tests in
//! `crabomination/src/tests/recent142.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, etb, lose_life, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, ZoneDest,
};
use crate::game::effects::food_token;
use crate::mana::{b, cost, g, generic, r, u, w, Color, SpendRestriction};

/// 1/1 black Rat token with "This token can't block."
fn rat_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}

fn sac_a_food() -> Option<(R, u32)> {
    Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 1))
}

// ── Black-Green / Black-Red ─────────────────────────────────────────────────────

/// Greta, Sweettooth Scourge — {1}{B}{G} 3/3 legendary Human Warrior. ETB Food;
/// {G}, Sac a Food: +1/+1 counter (sorcery); {1}{B}, Sac a Food: draw, lose 1.
pub fn greta_sweettooth_scourge() -> CardDefinition {
    CardDefinition {
        name: "Greta, Sweettooth Scourge",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: food_token(),
        })],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                sorcery_speed: true,
                sac_other_filter: sac_a_food(),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                sac_other_filter: sac_a_food(),
                effect: Effect::Seq(vec![draw(1), lose_life(1, Selector::You)]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Totentanz, Swarm Piper — {1}{B}{R} 2/3 legendary Human Warlock Bard. When it
/// or another nontoken creature you control dies, make a Rat. {1}{B}: an
/// attacking Rat you control gains deathtouch.
pub fn totentanz_swarm_piper() -> CardDefinition {
    CardDefinition {
        name: "Totentanz, Swarm Piper",
        cost: cost(&[generic(1), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: rat_token(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Rat)
                        .and(R::IsAttacking)
                        .and(R::ControlledByYou),
                ),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── White-Black ─────────────────────────────────────────────────────────────────

/// Neva, Stalked by Nightmares — {2}{W}{B} 2/2 legendary Human Noble with
/// menace. ETB return a creature/enchantment card from your graveyard; when an
/// enchantment you control dies, grow Neva and scry 1.
pub fn neva_stalked_by_nightmares() -> CardDefinition {
    CardDefinition {
        name: "Neva, Stalked by Nightmares",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(Effect::Move {
                what: target_filtered(R::Creature.or(R::Enchantment).and(R::InGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Enchantment,
                    }),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                ]),
            },
        ],
        ..Default::default()
    }
}

// ── Green-White ─────────────────────────────────────────────────────────────────

/// Syr Armont, the Redeemer — {3}{G}{W} 4/4 legendary Human Knight. ETB hang a
/// Monster Role on another creature you control; your enchanted creatures +1/+1.
pub fn syr_armont_the_redeemer() -> CardDefinition {
    CardDefinition {
        name: "Syr Armont, the Redeemer",
        cost: cost(&[generic(3), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::CreateTokenAttachedTo {
            target: target_filtered(
                R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            definition: super::woe_roles::monster_role(),
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creatures you control get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::IsEnchanted),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

// ── Green-Blue / Blue-Red ───────────────────────────────────────────────────────

/// Troyan, Gutsy Explorer — {1}{G}{U} 1/3 legendary Vedalken Scout. {T}: Add
/// {G}{U}, spendable only on MV-5+ or {X} spells. {U}, {T}: loot.
pub fn troyan_gutsy_explorer() -> CardDefinition {
    CardDefinition {
        name: "Troyan, Gutsy Explorer",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Green, Color::Blue])),
                        SpendRestriction::HighMvOrX,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[u()]),
                effect: Effect::Seq(vec![
                    draw(1),
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Johann, Apprentice Sorcerer — {2}{U}{R} 2/5 legendary Human Wizard Sorcerer.
/// Once each turn, you may cast an instant or sorcery spell from the top of your
/// library.
pub fn johann_apprentice_sorcerer() -> CardDefinition {
    CardDefinition {
        name: "Johann, Apprentice Sorcerer",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "Once each turn, cast an instant or sorcery from the top of your library.",
            effect: StaticEffect::PlayFromLibraryTopOncePerTurn {
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            },
        }],
        ..Default::default()
    }
}

// ── White enchantment / Blue instant ────────────────────────────────────────────

/// Solitary Sanctuary — {2}{W} Enchantment. ETB tap + stun an opponent's
/// creature; whenever you tap an untapped enemy creature, grow one of yours.
pub fn solitary_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Solitary Sanctuary",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::ONE,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::YouTapped)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    }),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Farsight Ritual — {2}{U}{U} Instant with Bargain. Dig four (eight if
/// bargained); put two into your hand and the rest on the bottom.
pub fn farsight_ritual() -> CardDefinition {
    let dig = |count: i32| Effect::LookPickToHand {
        who: PlayerRef::You,
        count: Value::Const(count),
        rest_to_graveyard: false,
        pick_filter: None,
        take: Some(Value::Const(2)),
        to_battlefield: false,
        gain_life_if_pick: None,
        gain_life_greatest_power_rest: false,
        optional: false,
        picked_lands_to_battlefield: false,
        rest_bottom_random: false,
    };
    CardDefinition {
        name: "Farsight Ritual",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Bargain],
        effect: Effect::If {
            cond: Predicate::SpellWasBargained,
            then: Box::new(dig(8)),
            else_: Box::new(dig(4)),
        },
        ..Default::default()
    }
}
