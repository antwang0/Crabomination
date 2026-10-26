//! Commander: 2026-10-19 (`016Xh16H`) — Narfi, Betrayer King's snow Zombies.
//! Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{ManaCost, ManaSymbol, b, cost, generic, u};

fn snow(n: usize) -> Vec<ManaSymbol> {
    vec![ManaSymbol::Snow; n]
}

fn with_snow(mut base: ManaCost, n: usize) -> ManaCost {
    base.symbols.extend(snow(n));
    base
}

fn snow_creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn snow_you_control() -> Value {
    Value::count(Selector::EachPermanent(R::IsSnow.and(R::ControlledByYou)))
}

/// Narfi, Betrayer King — {3}{U}{B} 4/3 legendary snow Zombie Wizard. Other
/// snow and Zombie creatures you control get +1/+1; {S}{S}{S}: return it from
/// your graveyard to the battlefield tapped.
pub fn narfi_betrayer_king() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary, Supertype::Snow],
        static_abilities: vec![StaticAbility {
            description: "Other snow and Zombie creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::IsSnow.or(R::HasCreatureType(CreatureType::Zombie)))
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost { symbols: snow(3) },
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..snow_creature(
            "Narfi, Betrayer King",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            4,
            3,
        )
    }
}

/// Icebind Pillar — {2}{U} snow artifact. {S}, {T}: tap target artifact or
/// creature.
pub fn icebind_pillar() -> CardDefinition {
    CardDefinition {
        name: "Icebind Pillar",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost { symbols: snow(1) },
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Artifact.or(R::Creature)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shimmerdrift Vale — snow land; enters tapped; as it enters choose a
/// color; {T}: one mana of that color.
pub fn shimmerdrift_vale() -> CardDefinition {
    CardDefinition {
        name: "Shimmerdrift Vale",
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Frostwalk Bastion — snow land. {T}: {C}. {1}{S}: until end of turn it's a
/// 2/3 Construct artifact creature (still a land). Dealing combat damage to a
/// creature taps it, and it doesn't untap during its controller's next untap
/// step.
pub fn frostwalk_bastion() -> CardDefinition {
    CardDefinition {
        name: "Frostwalk Bastion",
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: with_snow(cost(&[generic(1)]), 1),
                // "a 2/3 Construct artifact creature": the artifact type rides
                // the turn, as the colorless manlands' does.
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(2),
                        toughness: Value::Const(3),
                        creature_types: vec![CreatureType::Construct],
                        keywords: vec![],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::AddCardTypeIndefinitely { what: Selector::This, card_type: CardType::Artifact, until_eot: true },
                ]),
                ..Default::default()
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::SkipNextUntap { what: Selector::Target(0) },
            ]),
        }],
        ..Default::default()
    }
}

/// Frost Augur — {U} 1/2 snow Human Wizard. {S}, {T}: look at the top card
/// of your library; if it's a snow card, you may put it into your hand.
pub fn frost_augur() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost { symbols: snow(1) },
            tap_cost: true,
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: top(), filter: R::IsSnow },
                then: Box::new(Effect::MayDo {
                    description: "Reveal the snow card and put it into your hand?".into(),
                    body: Box::new(Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) }),
                }),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..snow_creature("Frost Augur", cost(&[u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 2)
    }
}

/// Grim Draugr — {2}{B} 3/2 snow Zombie Berserker. {1}{S}: +1/+0 and menace
/// until end of turn.
pub fn grim_draugr() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: with_snow(cost(&[generic(1)]), 1),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Menace, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..snow_creature(
            "Grim Draugr",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Berserker],
            3,
            2,
        )
    }
}

/// Priest of the Haunted Edge — {1}{B} 0/4 snow Zombie Cleric. {T},
/// sacrifice it: target creature gets -X/-X until end of turn, X the snow
/// lands you control; sorcery speed.
pub fn priest_of_the_haunted_edge() -> CardDefinition {
    let x = || Value::count(Selector::EachPermanent(R::IsSnow.and(R::Land).and(R::ControlledByYou)));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Negate(Box::new(x())),
                toughness: Value::Negate(Box::new(x())),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..snow_creature(
            "Priest of the Haunted Edge",
            cost(&[generic(1), b()]),
            vec![CreatureType::Zombie, CreatureType::Cleric],
            0,
            4,
        )
    }
}

/// Rimebound Dead — {B} 1/1 snow Skeleton. {S}: regenerate it.
pub fn rimebound_dead() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost { symbols: snow(1) },
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..snow_creature("Rimebound Dead", cost(&[b()]), vec![CreatureType::Skeleton], 1, 1)
    }
}

/// Winter's Rest — {1}{U} snow Aura. Entering, it taps the enchanted
/// creature; while you control another snow permanent, that creature
/// doesn't untap during its controller's untap step.
pub fn winters_rest() -> CardDefinition {
    CardDefinition {
        name: "Winter's Rest",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Tap { what: Selector::AttachedTo(Box::new(Selector::This)) })],
        static_abilities: vec![StaticAbility {
            description: "As long as you control another snow permanent, enchanted creature doesn't untap.",
            effect: StaticEffect::PreventUntapGlobal {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                condition: Some(Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(R::IsSnow.and(R::ControlledByYou).and(R::OtherThanSource))),
                    Value::ONE,
                )),
            },
        }],
        ..Default::default()
    }
}

/// Dead of Winter — {2}{B} sorcery. All nonsnow creatures get -X/-X until end
/// of turn, X the snow permanents you control.
pub fn dead_of_winter() -> CardDefinition {
    CardDefinition {
        name: "Dead of Winter",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::IsSnow.negate())),
            power: Value::Negate(Box::new(snow_you_control())),
            toughness: Value::Negate(Box::new(snow_you_control())),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}
