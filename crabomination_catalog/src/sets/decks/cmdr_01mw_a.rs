//! Commander: 2026-10-18 (`01MWhJbe`) closers, first batch. Tests in
//! `tests/recent_b/cmdr_01mw_a.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef};
use crate::mana::{cost, g, generic, r, u, w};
use std::sync::Arc;

/// Ixalli's Diviner — {1}{G} 0/3 Human Druid; when it enters, it explores.
pub fn ixallis_diviner() -> CardDefinition {
    CardDefinition {
        name: "Ixalli's Diviner",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Druid], ..Default::default() },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Explore { who: Selector::This })],
        ..Default::default()
    }
}

/// Origin of Iron Man — {4}{U} Saga. I: tap up to one target creature; it
/// doesn't untap while this Saga remains. II: draw two. III: you may put an
/// artifact card with mana value 5 or less from your hand onto the battlefield.
pub fn origin_of_iron_man() -> CardDefinition {
    CardDefinition {
        name: "Origin of Iron Man",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::TapAndLockWhileSourcePresent {
                        what: Selector::Target(0),
                        while_you_control: false,
                    }),
                },
            ),
            (2, Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
            (
                3,
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Artifact.and(R::ManaValueAtMost(5)),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
            ),
        ],
        ..Default::default()
    }
}

/// Odric, Blood-Cursed — {1}{R}{W} legendary 3/3 Vampire Soldier. When it
/// enters, create X Blood tokens, X = how many of twelve evergreen keywords
/// appear among creatures you control (each counted once).
pub fn odric_blood_cursed() -> CardDefinition {
    use crate::game::effects::blood_token;
    let has = |kw: Keyword| {
        Value::Min(
            Box::new(Value::count(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::HasKeyword(kw)),
            ))),
            Box::new(Value::ONE),
        )
    };
    let x = Value::Sum(
        [
            Keyword::Flying,
            Keyword::FirstStrike,
            Keyword::DoubleStrike,
            Keyword::Deathtouch,
            Keyword::Haste,
            Keyword::Hexproof,
            Keyword::Indestructible,
            Keyword::Lifelink,
            Keyword::Menace,
            Keyword::Reach,
            Keyword::Trample,
            Keyword::Vigilance,
        ]
        .into_iter()
        .map(has)
        .collect(),
    );
    CardDefinition {
        name: "Odric, Blood-Cursed",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire, CreatureType::Soldier], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: x,
            definition: Arc::new(blood_token()),
        })],
        ..Default::default()
    }
}

/// Kellan, Daring Traveler // Journey On — {1}{W} legendary 2/3 Human Faerie
/// Scout. Attacking reveals your top card: a creature card with mana value 3
/// or less goes to hand, anything else may be milled. Adventure {G} sorcery:
/// create 1 + (opponents who control an artifact) Map tokens.
pub fn kellan_daring_traveler() -> CardDefinition {
    use crate::game::effects::map_token;
    CardDefinition {
        name: "Kellan, Daring Traveler",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Faerie, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::RevealTopAndDrawIf {
                who: PlayerRef::You,
                reveal_filter: R::Creature.and(R::ManaValueAtMost(3)),
                may_graveyard_miss: true,
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Journey On",
            cost: cost(&[g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Sum(vec![
                    Value::ONE,
                    Value::OpponentsControllingAnyOf(Box::new(Selector::EachPermanent(R::Artifact))),
                ]),
                definition: Arc::new(map_token()),
            },
        })),
        ..Default::default()
    }
}

/// Invasion of Regatha // Disciples of the Inferno — {2}{R} Siege, defense 5.
/// ETB: 4 damage to another target battle or opponent and 1 damage to up to
/// one target creature. Back: 4/4 prowess Human Monk; a noncreature source you
/// control deals +2 to a creature, battle, or opponent.
pub fn invasion_of_regatha() -> CardDefinition {
    use crate::card::{BattleSubtype, StaticAbility, StaticEffect};
    let disciples = CardDefinition {
        name: "Disciples of the Inferno",
        card_types: vec![CardType::Creature],
        color_indicator: vec![crate::mana::Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Monk], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Prowess],
        static_abilities: vec![StaticAbility {
            description: "If a noncreature source you control would deal damage to a creature, battle, or opponent, it deals that much damage plus 2 instead.",
            effect: StaticEffect::NoncreatureSourceDamageBonus { amount: 2 },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Regatha",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 5,
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::HasCardType(CardType::Battle).and(R::OtherThanSource).or(R::OpponentPlayer),
                    },
                    amount: Value::Const(4),
                },
                Effect::DealDamage { to: Selector::TargetFiltered { slot: 1, filter: R::Creature }, amount: Value::ONE },
            ])),
        })],
        back_face: Some(Box::new(disciples)),
        ..Default::default()
    }
}

/// Thorin, Mountain-king — {3}{R} legendary 3/4 trample Dwarf Noble. When it
/// enters, attach any number of Equipment you control to target creature you
/// control; if one or more became attached, that creature deals damage equal
/// to its power to up to one target creature.
pub fn thorin_mountain_king() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Predicate};
    CardDefinition {
        name: "Thorin, Mountain-king",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Noble], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::AttachAnyNumberTo {
                    what: Selector::EachPermanent(
                        R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou),
                    ),
                    to: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::count(Selector::LastMoved), Value::ONE),
                    then: Box::new(Effect::DealDamageEqualToPower {
                        source: Selector::Target(0),
                        target: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Dwarven Hammer — {2}{R} Equipment: +3/+0 and trample; equip {3}. When it
/// enters, you may pay {2}: a 2/1 red Dwarf Berserker, then attach it to that.
pub fn dwarven_hammer() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus, TokenDefinition};
    let berserker = TokenDefinition {
        name: "Dwarf Berserker".into(),
        power: 2,
        toughness: 1,
        colors: vec![crate::mana::Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Berserker], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Dwarven Hammer",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 3, keywords: vec![Keyword::Trample], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {2} to create a 2/1 Dwarf Berserker and attach Dwarven Hammer to it?".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(berserker) },
                Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
            ])),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Bombur, Gentle Dreamer — {2}{R} legendary 5/3 Dwarf Bard; storied; it
/// doesn't untap during your untap step unless you have an enduring story.
pub fn bombur_gentle_dreamer() -> CardDefinition {
    use crate::card::{Predicate, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Bombur, Gentle Dreamer",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Bard], ..Default::default() },
        power: 5,
        toughness: 3,
        static_abilities: vec![
            StaticAbility {
                description: "Storied (If you control three or more artifacts, legendaries, and/or Sagas, you have an enduring story for the rest of the game.)",
                effect: StaticEffect::Storied,
            },
            StaticAbility {
                description: "Bombur doesn't untap during your untap step unless you have an enduring story.",
                effect: StaticEffect::PreventUntapGlobal {
                    applies_to: Selector::This,
                    condition: Some(Predicate::Not(Box::new(Predicate::HasEnduringStory { who: PlayerRef::You }))),
                },
            },
        ],
        ..Default::default()
    }
}

/// Iron Hills — land, enters tapped; {T}: add {R} or {W}. {2}{R}{W}, {T},
/// sacrifice it: two +1/+1 counters on target Dwarf you control; sorcery speed.
pub fn iron_hills() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::mana::Color;
    let mut d = crate::sets::dual_land_untyped("Iron Hills", Color::Red, Color::White, vec![]);
    d.static_abilities.push(crate::sets::enters_tapped());
    d.activated_abilities.push(ActivatedAbility {
        mana_cost: cost(&[generic(2), r(), w()]),
        tap_cost: true,
        sac_cost: true,
        sorcery_speed: true,
        effect: Effect::AddCounter {
            what: crate::effect::shortcut::target_filtered(
                R::Creature.and(R::HasCreatureType(CreatureType::Dwarf)).and(R::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        },
        ..Default::default()
    });
    d
}

/// Vow to Erebor — {1}{W} instant. Untap target creature you control; it gets
/// +2/+2 until end of turn; if it's a Dwarf, you may attach an Equipment you
/// control to it.
pub fn vow_to_erebor() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Predicate};
    use crate::effect::shortcut::{choose_some_then, chosen_one};
    use crate::effect::Duration;
    let it = || Selector::Target(0);
    CardDefinition {
        name: "Vow to Erebor",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap { what: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByYou)), up_to: None },
            Effect::PumpPT { what: it(), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::If {
                cond: Predicate::EntityMatches { what: it(), filter: R::HasCreatureType(CreatureType::Dwarf) },
                then: Box::new(choose_some_then(
                    Selector::EachPermanent(R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou)),
                    PlayerRef::You,
                    Value::ONE,
                    true,
                    Effect::Attach { what: chosen_one(), to: it() },
                )),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}
