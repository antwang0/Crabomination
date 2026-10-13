//! Commander: gaps of EDHREC's commanders ranked 1501-2000 (the 2026-10-13
//! twenty-page rescan), fifth file. Tests in `tests/recent_b/cmdr_rank2k_e.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, SplitCard, SplitHalf, StaticAbility, StaticEffect,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, LookPick, PlayerRef};
use crate::mana::{b, cost, g, generic, u, Color, ManaCost};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![crate::card::Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn merfolk() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Merfolk))
}

/// A token copy of `source` you control.
fn copy_of(source: Selector) -> Effect {
    Effect::CreateTokenCopyOf {
        extra_keywords: vec![],
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        no_mana_cost: false,
        enters_with_counters: None,
        remove_keywords: vec![],
    }
}

/// Destined // Lead — Destined {1}{B} instant: +1/+0 and indestructible until
/// end of turn. Lead {3}{G} sorcery, aftermath: every creature able to block
/// the target creature this turn does so.
pub fn destined_lead() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::AllMustBlock,
                    duration: Duration::EndOfTurn,
                },
            },
            fuse: false,
            aftermath: true,
        })),
        ..spell(
            "Destined // Lead",
            cost(&[generic(1), b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Ellyn Harbreeze, Busybody — {3}{W} legendary 2/4 Human Peasant. {T}: look
/// at the top X cards (X = tokens you created this turn), one to hand, the
/// rest on the bottom at random. Choose a Background.
pub fn ellyn_harbreeze_busybody() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::TokensCreatedThisTurn(PlayerRef::You),
                rest_bottom_random: true,
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..legend(
            "Ellyn Harbreeze, Busybody",
            cost(&[generic(3), crate::mana::w()]),
            vec![CreatureType::Human, CreatureType::Peasant],
            2,
            4,
        )
    }
}

/// Mirage Mockery — {2}{U} sorcery, choose one: a token copy of an artifact
/// creature you control, or of a nonartifact creature you control. Entwine
/// {2}{U}.
pub fn mirage_mockery() -> CardDefinition {
    let yours = |filter: R| target_filtered(R::Creature.and(filter).and(R::ControlledByYou));
    CardDefinition {
        keywords: vec![Keyword::Entwine(cost(&[generic(2), u()]))],
        ..spell(
            "Mirage Mockery",
            cost(&[generic(2), u()]),
            CardType::Sorcery,
            Effect::ChooseMode(vec![copy_of(yours(R::Artifact)), copy_of(yours(R::Artifact.negate()))]),
        )
    }
}

/// Mirror Mockery — {1}{U} Aura: whenever the enchanted creature attacks, you
/// may make a token copy of it, exiled at end of combat.
pub fn mirror_mockery() -> CardDefinition {
    CardDefinition {
        name: "Mirror Mockery",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Create a token copy of the enchanted creature?".into(),
                    body: Box::new(Effect::Seq(vec![
                        copy_of(Selector::This),
                        Effect::StampTokenCopyExceptions {
                            artifact_subtypes: vec![],
                            activated: vec![],
                            triggered: vec![TriggeredAbility {
                                event: EventSpec::new(
                                    EventKind::StepBegins(crate::game::types::TurnStep::EndCombat),
                                    EventScope::AnyPlayer,
                                ),
                                effect: Effect::Exile { what: Selector::This },
                            }],
                        },
                    ])),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Namor the Sub-Mariner — {1}{U}{U} legendary */4 flying Mutant Merfolk
/// Villain; power is the Merfolk you control. A noncreature spell you cast
/// with blue pips makes that many 1/1 blue Merfolk.
pub fn namor_the_sub_mariner() -> CardDefinition {
    let folk = TokenDefinition {
        name: "Merfolk".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Blue],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Namor's power is equal to the number of Merfolk you control.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::CountOf(Box::new(Selector::EachPermanent(merfolk().and(R::ControlledByYou)))),
                toughness: Value::Const(4),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                crate::effect::shortcut::cast_is_noncreature(),
                Predicate::ValueAtLeast(Value::CastSpellPipsOf(Color::Blue), Value::ONE),
            ])),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CastSpellPipsOf(Color::Blue),
                definition: Arc::new(folk),
            },
        }],
        ..legend(
            "Namor the Sub-Mariner",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Mutant, CreatureType::Merfolk, CreatureType::Villain],
            0,
            4,
        )
    }
}

/// Ochran Assassin — {1}{B}{G} 1/1 deathtouch Elf Assassin; every creature
/// able to block it does so.
pub fn ochran_assassin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::AllMustBlock],
        ..creature("Ochran Assassin", cost(&[generic(1), b(), g()]), vec![CreatureType::Elf, CreatureType::Assassin], 1, 1)
    }
}

/// River Sneak — {1}{U} 1/1 unblockable Merfolk Warrior; another Merfolk of
/// yours entering gives it +1/+1 until end of turn.
pub fn river_sneak() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: merfolk() }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("River Sneak", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Warrior], 1, 1)
    }
}

/// Wanderwine Prophets — {4}{U}{U} 4/4 Merfolk Wizard. Champion a Merfolk.
/// Its combat damage to a player may sacrifice a Merfolk for an extra turn.
pub fn wanderwine_prophets() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Champion { filter: R::HasCreatureType(CreatureType::Merfolk) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MaySacrifice {
                    description: "Sacrifice a Merfolk to take an extra turn?".into(),
                    filter: merfolk().and(R::ControlledByYou),
                    count: Value::ONE,
                    then: Box::new(Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE }),
                    else_: None,
                },
            },
        ],
        ..creature(
            "Wanderwine Prophets",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            4,
            4,
        )
    }
}
