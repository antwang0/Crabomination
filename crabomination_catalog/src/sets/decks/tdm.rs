//! Tarkir: Dragonstorm (TDM) gap batch — commons/uncommons on existing
//! primitives: combat tricks (Alesha's Legacy), flash Auras (Fire-Rim Form,
//! Fresh Start, Wingspan Stride, Ringing Strike Mastery), graveyard hate
//! (Jade-Cast Sentinel), a dig-and-mill body (Gurmag Nightwatch), MV-gated
//! removal (Kin-Tree Severance), counter distribution (Armament Dragon), a
//! graveyard-return + power-sling (Lie in Wait), a Dragon enter-counter rock
//! (Dragonstorm Globe), modal spells (Riverwalk Technique, Seize Opportunity,
//! Rally the Monastery), and an O-Ring (Static Snare). Tests in
//! `crabomination/src/tests/tdm.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, mobilize, target_filtered};
use crate::effect::{
    AttackingTokenCleanup, Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition,
    ManaPayload, PlayerRef, Selector, ZoneDest,
};
use crate::mana::{b, cost, g, generic, mono_hybrid, r, u, w, x, Color};

/// Alesha's Legacy — {1}{B} Instant. Target creature you control gains
/// deathtouch and indestructible until end of turn.
pub fn aleshas_legacy() -> CardDefinition {
    CardDefinition {
        name: "Alesha's Legacy",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Fire-Rim Form — {1}{R} Aura, Flash. Enchant creature. When it enters,
/// enchanted creature gains first strike until end of turn. Enchanted creature
/// gets +2/+0.
pub fn fire_rim_form() -> CardDefinition {
    CardDefinition {
        name: "Fire-Rim Form",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus { power: 2, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            keyword: Keyword::FirstStrike,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Jade-Cast Sentinel — {4} Artifact Creature — Ape Snake 1/5, Reach. {2}, {T}:
/// Put target card from a graveyard on the bottom of its owner's library.
pub fn jade_cast_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Jade-Cast Sentinel",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape, CreatureType::Snake],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gurmag Nightwatch — {2/B}{2/G}{2/U} 3/3 Human Ranger. When it enters, look at
/// the top three cards of your library, put one back on top, the rest into your
/// graveyard.
pub fn gurmag_nightwatch() -> CardDefinition {
    CardDefinition {
        name: "Gurmag Nightwatch",
        cost: cost(&[
            mono_hybrid(2, Color::Black),
            mono_hybrid(2, Color::Green),
            mono_hybrid(2, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ranger],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::LookTopKeepOneRestToGraveyard {
            count: Value::Const(3),
            who: None,
            exile_rest: false,
        })],
        ..Default::default()
    }
}

/// Kin-Tree Severance — {2/W}{2/B}{2/G} Instant. Exile target permanent with
/// mana value 3 or greater.
pub fn kin_tree_severance() -> CardDefinition {
    CardDefinition {
        name: "Kin-Tree Severance",
        cost: cost(&[
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::Black),
            mono_hybrid(2, Color::Green),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(R::Permanent.and(R::ManaValueAtLeast(3))),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Armament Dragon — {3}{W}{B}{G} 3/4 Dragon, Flying. When it enters, distribute
/// three +1/+1 counters among one, two, or three target creatures you control.
pub fn armament_dragon() -> CardDefinition {
    CardDefinition {
        name: "Armament Dragon",
        cost: cost(&[generic(3), w(), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DistributeCounters {
            total: Value::Const(3),
            counter: CounterType::PlusOnePlusOne,
            filter: R::Creature.and(R::ControlledByYou),
            max_targets: 3,
        })],
        ..Default::default()
    }
}

/// Fresh Start — {1}{U} Aura, Flash. Enchant creature. Enchanted creature gets
/// −5/−0 and loses all abilities.
pub fn fresh_start() -> CardDefinition {
    CardDefinition {
        name: "Fresh Start",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: -5,
            remove_abilities: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Lie in Wait — {B}{G}{U} Sorcery. Return target creature card from your
/// graveyard to your hand. It deals damage equal to that card's power to target
/// creature.
pub fn lie_in_wait() -> CardDefinition {
    CardDefinition {
        name: "Lie in Wait",
        cost: cost(&[b(), g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::InYourGraveyard),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Dragonstorm Globe — {3} Artifact. Each Dragon you control enters with an
/// additional +1/+1 counter on it. {T}: Add one mana of any color.
pub fn dragonstorm_globe() -> CardDefinition {
    CardDefinition {
        name: "Dragonstorm Globe",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Each Dragon you control enters with an additional +1/+1 counter on it.",
            effect: StaticEffect::TypeEntersWithCounter {
                creature_type: CreatureType::Dragon,
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColors(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wingspan Stride — {U} Aura. Enchant creature. Enchanted creature gets +1/+1
/// and has flying. {2}{U}: Return this Aura to its owner's hand.
pub fn wingspan_stride() -> CardDefinition {
    CardDefinition {
        name: "Wingspan Stride",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Riverwalk Technique — {3}{U} Instant. Choose one — the owner of target nonland
/// permanent puts it on their choice of the top or bottom of their library; or
/// counter target noncreature spell.
pub fn riverwalk_technique() -> CardDefinition {
    CardDefinition {
        name: "Riverwalk Technique",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: Selector::TargetFiltered { slot: 0, filter: R::Nonland },
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::OwnerChoice,
                },
            },
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::Creature.negate())),
            },
        ]),
        ..Default::default()
    }
}

/// Static Snare — {4}{W} Enchantment, Flash. When it enters, exile target
/// artifact or creature an opponent controls until this enchantment leaves.
/// (The "costs {1} less per attacking creature" reduction is dropped.)
pub fn static_snare() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Static Snare",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered((R::Artifact.or(R::Creature)).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Seize Opportunity — {2}{R} Instant. Choose one — exile the top two cards of
/// your library and play them until the end of your next turn; or up to two
/// target creatures each get +2/+1 until end of turn.
pub fn seize_opportunity() -> CardDefinition {
    CardDefinition {
        name: "Seize Opportunity",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Ringing Strike Mastery — {U} Aura. Enchant creature. When it enters, tap the
/// enchanted creature. Enchanted creature doesn't untap during its controller's
/// untap step. (The granted "{5}: untap this creature" is dropped.)
pub fn ringing_strike_mastery() -> CardDefinition {
    CardDefinition {
        name: "Ringing Strike Mastery",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature doesn't untap during its controller's untap step.",
                effect: StaticEffect::PreventUntap {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                },
            },
            StaticAbility {
                description: "Enchanted creature has \"{5}: Untap this creature.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                    ability: ActivatedAbility {
                        mana_cost: cost(&[generic(5)]),
                        effect: Effect::Untap { what: Selector::This, up_to: None },
                        ..Default::default()
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Krumar Initiate — {1}{B} 2/2 Human Cleric. {X}{B}, {T}, Pay X life:
/// This creature endures X. Activate only as a sorcery.
pub fn krumar_initiate() -> CardDefinition {
    CardDefinition {
        name: "Krumar Initiate",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), b()]),
            tap_cost: true,
            x_life_cost: true,
            sorcery_speed: true,
            effect: Effect::Endure { target: Selector::This, n: Value::XFromCost },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Zurgo's Vanguard — {2}{R} */3 Dog Soldier with Mobilize 1. Its power
/// equals the number of creatures you control.
pub fn zurgos_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Zurgo's Vanguard",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Soldier],
            ..Default::default()
        },
        toughness: 3,
        dynamic_pt: Some(DynamicPt::CreaturesControlledPower { base_p: 0, base_t: 3 }),
        triggered_abilities: vec![mobilize(1)],
        ..Default::default()
    }
}

/// War Effort — {3}{R} Enchantment. Creatures you control get +1/+0. Whenever
/// you attack, create a 1/1 red Warrior token that's tapped and attacking,
/// sacrificed at end of combat (Mobilize).
pub fn war_effort() -> CardDefinition {
    CardDefinition {
        name: "War Effort",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Warrior".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Warrior],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                cleanup: AttackingTokenCleanup::SacrificeAtEndOfCombat,
            },
        }],
        ..Default::default()
    }
}

/// Dragon's Prey — {2}{B} Instant. Costs {2} more to cast if it targets a
/// Dragon. Destroy target creature.
pub fn dragons_prey() -> CardDefinition {
    CardDefinition {
        name: "Dragon's Prey",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        cost_increase_if_targets: Some((R::HasCreatureType(CreatureType::Dragon), 2)),
        effect: Effect::Destroy { what: target_filtered(R::Creature) },
        ..Default::default()
    }
}

fn white_monk_prowess_token() -> TokenDefinition {
    TokenDefinition {
        name: "Monk".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Monk], ..Default::default() },
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    }
}

/// Rally the Monastery — {3}{W} Instant, {2} less if you've cast another spell
/// this turn. Choose one — create two 1/1 white Monk tokens with prowess; up to
/// two target creatures you control each get +2/+2 until end of turn; or destroy
/// target creature with power 4 or greater.
pub fn rally_the_monastery() -> CardDefinition {
    CardDefinition {
        name: "Rally the Monastery",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_cast_spell: Some(2),
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: white_monk_prowess_token(),
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                filter: R::Creature.and(R::ControlledByYou),
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::PowerAtLeast(4))),
            },
        ]),
        ..Default::default()
    }
}
