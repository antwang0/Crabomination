//! Tarkir: Dragonstorm (TDM) gap batch — commons/uncommons on existing
//! primitives: combat tricks (Alesha's Legacy), flash Auras (Fire-Rim Form,
//! Fresh Start, Wingspan Stride, Ringing Strike Mastery), graveyard hate
//! (Jade-Cast Sentinel), a dig-and-mill body (Gurmag Nightwatch), MV-gated
//! removal (Kin-Tree Severance), counter distribution (Armament Dragon), a
//! graveyard-return + power-sling (Lie in Wait), a Dragon enter-counter rock
//! (Dragonstorm Globe), modal spells (Riverwalk Technique, Seize Opportunity,
//! Rally the Monastery), and an O-Ring (Static Snare). Batch 2 adds sieges /
//! dragonstorms / equipment: Salt Road Skirmish (destroy + transient Warriors),
//! Corroding Dragonstorm (drain/surveil + Dragon-enter self-bounce), Essence
//! Anchor (surveil rock gated on a graveyard departure), Stormbeacon Blade
//! (equip + mass-attack draw), plus the five gap completions (Krumar Initiate,
//! Zurgo's Vanguard, War Effort, Dragon's Prey, Ringing Strike Mastery's granted
//! untap). Batch 3 adds two Dragons (Jeskai Shrinekeeper, Kheru Goldkeeper),
//! Encroaching Dragonstorm (basic ramp + Dragon bounce) and Dragonclaw Strike
//! (double-P/T fight). Batch 4 adds Clarion Conqueror (activation lock),
//! Ambling Stormshell (Ward Turtle) and Furious Forebear (graveyard recursion).
//! Batch 5 adds Bewilder and Sarkhan, Dragon Ascendant. The Siege cycle
//! (Barrensteppe / Frostcliff / Glacierwood / Hollowmurk) rides the new
//! `CardDefinition.enter_modes` persistent as-enters mode choice. A later batch
//! adds Nature's Rhythm (X-search-to-battlefield + Harmonize), Smile at Death
//! (upkeep up-to-two graveyard reanimate via `ApplyToTargets`), Roar of
//! Endless Song (Saga — Elephants then a team P/T double via `ForEach`), Zurgo
//! (Mobilize 2), Rot-Curse Rakshasa (5/5 trample decayed), and Flamehold
//! Grappler (ETB copy-next-spell). Tests in `crabomination/src/tests/tdm.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EnchantmentSubtype, EnterMode, EquipBonus, Keyword, LandType, MayPlayDuration,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    cast_is_instant_or_sorcery, drain, etb, mobilize, on_you_attack, target_any, target_filtered,
};
use crate::game::TurnStep;
use crate::effect::{
    AttackingTokenCleanup, Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition,
    ManaPayload, PlayerRef, Predicate, Selector, ZoneDest,
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

/// Static Snare — {4}{W} Enchantment, Flash. Costs {1} less per attacking
/// creature. When it enters, exile target artifact or creature an opponent
/// controls until this enchantment leaves.
pub fn static_snare() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Static Snare",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        affinity_filter: Some(R::Creature.and(R::IsAttacking)),
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

// ── TDM batch 2: sieges / dragonstorms / equipment (modern_decks) ──────────

fn warrior_haste_token() -> TokenDefinition {
    TokenDefinition {
        name: "Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Salt Road Skirmish — {3}{B} Sorcery. Destroy target creature. Create two
/// 1/1 red Warriors with haste, sacrificed at the beginning of the next end step.
pub fn salt_road_skirmish() -> CardDefinition {
    CardDefinition {
        name: "Salt Road Skirmish",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: warrior_haste_token(),
            },
            Effect::SacrificeLastCreatedTokensAtNextEndStep,
        ]),
        ..Default::default()
    }
}

/// Corroding Dragonstorm — {1}{B} Enchantment. ETB: each opponent loses 2 life,
/// you gain 2, surveil 2. When a Dragon you control enters, return this to hand.
pub fn corroding_dragonstorm() -> CardDefinition {
    CardDefinition {
        name: "Corroding Dragonstorm",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                drain(2),
                Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Dragon),
                    }),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
        ],
        ..Default::default()
    }
}

/// Essence Anchor — {2}{U} Artifact. Upkeep: surveil 1. {T}: create a 2/2 black
/// Zombie Druid — only if a card left your graveyard this turn.
pub fn essence_anchor() -> CardDefinition {
    CardDefinition {
        name: "Essence Anchor",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::CardsLeftGraveyardThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::Const(1),
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Zombie Druid".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Zombie, CreatureType::Druid],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stormbeacon Blade — {1}{W} Equipment. Equipped creature gets +3/+0 and, when
/// it attacks, its controller draws a card if they control three or more
/// attacking creatures. Equip {2}.
pub fn stormbeacon_blade() -> CardDefinition {
    CardDefinition {
        name: "Stormbeacon Blade",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 0,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: 3 },
                    then: Box::new(Effect::Draw {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── TDM batch 3: Dragons / dragonstorm ramp / wedge sorceries ──────────────

/// Jeskai Shrinekeeper — {2}{U}{R}{W} 3/3 Dragon with flying and haste.
/// Whenever it deals combat damage to a player, you gain 1 life and draw a card.
pub fn jeskai_shrinekeeper() -> CardDefinition {
    CardDefinition {
        name: "Jeskai Shrinekeeper",
        cost: cost(&[generic(2), u(), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Encroaching Dragonstorm — {3}{G} Enchantment. ETB: search up to two basic
/// lands onto the battlefield tapped. When a Dragon you control enters, return
/// this to its owner's hand.
pub fn encroaching_dragonstorm() -> CardDefinition {
    CardDefinition {
        name: "Encroaching Dragonstorm",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(2),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Dragon),
                    }),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            },
        ],
        ..Default::default()
    }
}

/// Kheru Goldkeeper — {1}{B}{G}{U} 3/3 Dragon with flying. Whenever one or more
/// cards leave your graveyard during your turn, create a Treasure. Renew —
/// {2}{B}{G}{U}, Exile this from your graveyard: put two +1/+1 counters and a
/// flying counter on target creature. Sorcery speed.
pub fn kheru_goldkeeper() -> CardDefinition {
    CardDefinition {
        name: "Kheru Goldkeeper",
        cost: cost(&[generic(1), b(), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                kind: EventKind::CardLeftGraveyard,
                scope: EventScope::YourControl,
                filter: Some(Predicate::IsTurnOf(PlayerRef::You)),
                once_per_turn: true,
                per_subject_cap: None,
                actor_is_opponent: false,
            },
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), g(), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::AddKeywordCounter {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dragonclaw Strike — {2/G}{2/U}{2/R} Sorcery. Double target creature you
/// control's power and toughness until end of turn, then it fights up to one
/// target creature an opponent controls.
pub fn dragonclaw_strike() -> CardDefinition {
    CardDefinition {
        name: "Dragonclaw Strike",
        cost: cost(&[
            mono_hybrid(2, Color::Green),
            mono_hybrid(2, Color::Blue),
            mono_hybrid(2, Color::Red),
        ]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                power: Value::PowerOf(Box::new(Selector::Target(0))),
                toughness: Value::ToughnessOf(Box::new(Selector::Target(0))),
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
        ..Default::default()
    }
}

// ── TDM batch 4: activation lock, big Turtle, graveyard recursion ──────────

/// Clarion Conqueror — {2}{W} 3/3 Dragon with flying. Activated abilities of
/// artifacts, creatures, and planeswalkers can't be activated (mana abilities
/// still work; the planeswalker/loyalty half rides the artifact+creature locks).
pub fn clarion_conqueror() -> CardDefinition {
    CardDefinition {
        name: "Clarion Conqueror",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Activated abilities of artifacts can't be activated.",
                effect: StaticEffect::ArtifactActivatedAbilitiesLocked,
            },
            StaticAbility {
                description: "Activated abilities of creatures can't be activated.",
                effect: StaticEffect::CreatureActivatedAbilitiesLocked,
            },
        ],
        ..Default::default()
    }
}

/// Ambling Stormshell — {3}{U}{U} 5/9 Turtle with Ward {2}. Whenever it attacks,
/// put three stun counters on it and draw three cards. Whenever you cast a
/// Turtle spell, untap it.
pub fn ambling_stormshell() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Ambling Stormshell",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Turtle], ..Default::default() },
        power: 5,
        toughness: 9,
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Stun,
                        amount: Value::Const(3),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Turtle),
                    },
                ),
                effect: Effect::Untap { what: Selector::This, up_to: None },
            },
        ],
        ..Default::default()
    }
}

/// Furious Forebear — {1}{W} 3/1 Spirit Warrior. Whenever a creature you control
/// dies while this is in your graveyard, you may pay {1}{W} to return it to hand.
pub fn furious_forebear() -> CardDefinition {
    CardDefinition {
        name: "Furious Forebear",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::FromYourGraveyard)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ControlledByYou),
                }),
            effect: Effect::MayPay {
                description: "Pay {1}{W}: return Furious Forebear from your graveyard to hand".into(),
                mana_cost: cost(&[generic(1), w()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

// ── TDM batch 5: tempo instant, untap engine, behold-Dragon commander ──────

/// Bewilder — {2}{U} Instant. Target creature gets −3/−0 until end of turn.
/// Draw a card.
pub fn bewilder() -> CardDefinition {
    CardDefinition {
        name: "Bewilder",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Sarkhan, Dragon Ascendant — {1}{R} 2/2 Human Druid. ETB: if you control a
/// Dragon (our "behold" proxy), create a Treasure. Whenever a Dragon you
/// control enters, put a +1/+1 counter on Sarkhan; until end of turn it becomes
/// a Dragon in addition to its other types and gains flying.
pub fn sarkhan_dragon_ascendant() -> CardDefinition {
    CardDefinition {
        name: "Sarkhan, Dragon Ascendant",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Dragon).and(R::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Dragon),
                    }),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::AddCreatureTypes {
                        what: Selector::This,
                        creature_types: vec![CreatureType::Dragon],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

// ── TDM batch 6: keyword beater, stun+impulse, mass shrink ─────────────────

/// Jeskai Brushmaster — {1}{U}{R}{W} 2/4 Orc Monk with double strike and prowess.
pub fn jeskai_brushmaster() -> CardDefinition {
    CardDefinition {
        name: "Jeskai Brushmaster",
        cost: cost(&[generic(1), u(), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::DoubleStrike, Keyword::Prowess],
        ..Default::default()
    }
}

/// Riverwheel Sweep — {2/U}{2/R}{2/W} Sorcery. Tap target creature and put three
/// stun counters on it. Then exile the top two cards of your library; until the
/// end of your next turn you may play them.
pub fn riverwheel_sweep() -> CardDefinition {
    CardDefinition {
        name: "Riverwheel Sweep",
        cost: cost(&[
            mono_hybrid(2, Color::Blue),
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::White),
        ]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(3),
            },
            // "Choose one of them" collapses to a may-play grant on both — a
            // strictly-better approximation matching the Tectonic Giant impulse.
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        ]),
        ..Default::default()
    }
}

/// Flowstone Slide — {X}{2}{R}{R} Sorcery. All creatures get +X/−X until end of
/// turn.
pub fn flowstone_slide() -> CardDefinition {
    CardDefinition {
        name: "Flowstone Slide",
        cost: cost(&[x(), generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::XFromCost,
            toughness: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── TDM batch 7: five-color relic ──────────────────────────────────────────

/// Dragonbroods' Relic — {1}{G} Artifact. {T}, Tap an untapped creature you
/// control: Add one mana of any color. {3}{W}{U}{B}{R}{G}, Sacrifice this:
/// Create a 4/4 all-color Dragon token named Reliquary Dragon with flying,
/// lifelink, and "When this token enters, it deals 3 damage to any target."
/// Sorcery speed.
pub fn dragonbroods_relic() -> CardDefinition {
    let reliquary_dragon = TokenDefinition {
        name: "Reliquary Dragon".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Blue, Color::Black, Color::Red, Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_any(),
            amount: Value::Const(3),
        })],
        ..Default::default()
    };
    CardDefinition {
        name: "Dragonbroods' Relic",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), w(), u(), b(), r(), g()]),
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: reliquary_dragon,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── TDM batch 8: becomes-tapped dig ────────────────────────────────────────

/// Traveling Botanist — {1}{G} 2/3 Dog Scout. Whenever it becomes tapped, look
/// at the top card of your library; if it's a land, put it into your hand,
/// otherwise you may bin it (both printed "may"s auto-taken).
pub fn traveling_botanist() -> CardDefinition {
    CardDefinition {
        name: "Traveling Botanist",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::LookTopLandToHandElseBin { who: PlayerRef::You },
        }],
        ..Default::default()
    }
}

/// "Creatures you control" — the shared anthem/counter selector for the Siege
/// cycle's mode abilities.
fn your_creatures() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou))
}

/// Barrensteppe Siege — {2}{W}{B} Enchantment. As it enters, choose Abzan or
/// Mardu. Abzan: at your end step, +1/+1 counter on each creature you control.
/// Mardu: at your end step, if a creature died under your control this turn,
/// each opponent sacrifices a creature.
pub fn barrensteppe_siege() -> CardDefinition {
    let end_step = || EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer);
    CardDefinition {
        name: "Barrensteppe Siege",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Abzan",
                triggered_abilities: vec![TriggeredAbility {
                    event: end_step(),
                    effect: Effect::AddCounter {
                        what: your_creatures(),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                }],
                ..Default::default()
            },
            EnterMode {
                label: "Mardu",
                triggered_abilities: vec![TriggeredAbility {
                    event: end_step(),
                    effect: Effect::If {
                        cond: Predicate::CreaturesDiedThisTurnAtLeast {
                            who: PlayerRef::You,
                            at_least: Value::ONE,
                        },
                        then: Box::new(Effect::Sacrifice {
                            who: Selector::Player(PlayerRef::EachOpponent),
                            count: Value::ONE,
                            filter: R::Creature,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                }],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Frostcliff Siege — {1}{U}{R} Enchantment. As it enters, choose Jeskai or
/// Temur. Jeskai: draw when your creatures deal combat damage to a player.
/// Temur: creatures you control get +1/+0 and have trample and haste.
pub fn frostcliff_siege() -> CardDefinition {
    CardDefinition {
        name: "Frostcliff Siege",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Jeskai",
                // `once_per_turn` approximates "one or more creatures … deal
                // combat damage" — the per-creature dealer event would over-draw.
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToPlayer,
                        EventScope::YourControl,
                    )
                    .once_per_turn(),
                    effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                }],
                ..Default::default()
            },
            EnterMode {
                label: "Temur",
                static_abilities: vec![
                    StaticAbility {
                        description: "Creatures you control get +1/+0.",
                        effect: StaticEffect::PumpPT {
                            applies_to: your_creatures(),
                            power: 1,
                            toughness: 0,
                        },
                    },
                    StaticAbility {
                        description: "Creatures you control have trample.",
                        effect: StaticEffect::GrantKeyword {
                            applies_to: your_creatures(),
                            keyword: Keyword::Trample,
                        },
                    },
                    StaticAbility {
                        description: "Creatures you control have haste.",
                        effect: StaticEffect::GrantKeyword {
                            applies_to: your_creatures(),
                            keyword: Keyword::Haste,
                        },
                    },
                ],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Glacierwood Siege — {1}{G}{U} Enchantment. As it enters, choose Temur or
/// Sultai. Temur: target player mills four when you cast an instant/sorcery.
/// Sultai: you may play lands from your graveyard.
pub fn glacierwood_siege() -> CardDefinition {
    CardDefinition {
        name: "Glacierwood Siege",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Temur",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                        .with_filter(cast_is_instant_or_sorcery()),
                    effect: Effect::Mill {
                        who: target_filtered(R::Player),
                        amount: Value::Const(4),
                    },
                }],
                ..Default::default()
            },
            EnterMode {
                label: "Sultai",
                static_abilities: vec![StaticAbility {
                    description: "You may play lands from your graveyard.",
                    effect: StaticEffect::MayPlayLandsFromGraveyard,
                }],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Hollowmurk Siege — {B}{G} Enchantment. As it enters, choose Sultai or Abzan.
/// Sultai: draw the first time each turn a counter is put on a creature you
/// control. Abzan: when you attack, +1/+1 counter on target attacker + menace.
pub fn hollowmurk_siege() -> CardDefinition {
    CardDefinition {
        name: "Hollowmurk Siege",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Sultai",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::AnyCounterAdded, EventScope::YourControl)
                        .with_filter(Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::Creature,
                        })
                        .once_per_turn(),
                    effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                }],
                ..Default::default()
            },
            EnterMode {
                label: "Abzan",
                triggered_abilities: vec![on_you_attack(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Creature.and(R::IsAttacking)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Menace,
                        duration: Duration::EndOfTurn,
                    },
                ]))],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Abzan Monument — {2} Artifact. ETB: search for a basic Plains/Swamp/Forest,
/// to hand. {1}{W}{B}{G}, {T}, Sac: create an X/X white Spirit, X = greatest
/// toughness among creatures you control (sorcery speed).
pub fn abzan_monument() -> CardDefinition {
    let greatest_toughness =
        || Value::ToughnessOf(Box::new(Selector::GreatestToughnessYouControl));
    CardDefinition {
        name: "Abzan Monument",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand.and(
                R::HasLandType(LandType::Plains)
                    .or(R::HasLandType(LandType::Swamp))
                    .or(R::HasLandType(LandType::Forest)),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            mana_cost: cost(&[generic(1), w(), b(), g()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Spirit".into(),
                    power: 0,
                    toughness: 0,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Spirit],
                        ..Default::default()
                    },
                    dynamic_pt: Some((greatest_toughness(), greatest_toughness())),
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Breaching Dragonstorm — {4}{R} Enchantment. ETB: exile from the top until a
/// nonland card; cast it free if its mana value is 8 or less, else it goes to
/// hand. When a Dragon you control enters, bounce this to hand.
pub fn breaching_dragonstorm() -> CardDefinition {
    CardDefinition {
        name: "Breaching Dragonstorm",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::ExileTopUntilNonlandMayPlay {
                who: PlayerRef::You,
                duration: MayPlayDuration::EndOfThisTurn,
                free: true,
                hand_unless_mv_below: Some(Value::Const(9)),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Dragon),
                    }),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
        ],
        ..Default::default()
    }
}

/// Dragonstorm Forecaster — {U} 0/3 Human Scout. {2}, {T}: search for a card
/// named Dragonstorm Globe or Boulderborn Dragon, to hand.
pub fn dragonstorm_forecaster() -> CardDefinition {
    CardDefinition {
        name: "Dragonstorm Forecaster",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasName("Dragonstorm Globe".into())
                    .or(R::HasName("Boulderborn Dragon".into())),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hundred-Battle Veteran — {3}{B} 4/2 Zombie Warrior. +2/+4 while three or more
/// kinds of counters are among your creatures. You may cast it from your
/// graveyard; if you do, it enters with a finality counter.
pub fn hundred_battle_veteran() -> CardDefinition {
    CardDefinition {
        name: "Hundred-Battle Veteran",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        static_abilities: vec![
            StaticAbility {
                description: "As long as there are three or more different kinds of counters among creatures you control, this creature gets +2/+4.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::DistinctCounterKindsAmongCreaturesAtLeast {
                        who: PlayerRef::You,
                        at_least: 3,
                    },
                    power: 2,
                    toughness: 4,
                    keywords: vec![],
                },
            },
            StaticAbility {
                description: "You may cast this card from your graveyard. If you do, it enters with a finality counter on it.",
                effect: StaticEffect::GraveyardCastWithLifeSurcharge {
                    filter: R::HasName("Hundred-Battle Veteran".into()),
                    life: 0,
                },
            },
        ],
        ..Default::default()
    }
}

/// Anafenza, Unyielding Lineage — {2}{W} 2/2 Spirit Soldier, Flash, First
/// strike. Whenever another nontoken creature you control dies, endure 2.
pub fn anafenza_unyielding_lineage() -> CardDefinition {
    CardDefinition {
        name: "Anafenza, Unyielding Lineage",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                }),
            effect: Effect::Endure { target: Selector::This, n: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Felothar, Dawn of the Abzan — {W}{B}{G} 3/3 Human Warrior, trample. Whenever
/// it enters or attacks, you may sacrifice a nonland permanent; if you do, put a
/// +1/+1 counter on each creature you control.
pub fn felothar_dawn_of_the_abzan() -> CardDefinition {
    let body = || Effect::MaySacrifice {
        description: "Sacrifice a nonland permanent?".into(),
        filter: R::Nonland,
        count: Value::ONE,
        then: Box::new(Effect::AddCounter {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        }),
        else_: None,
    };
    CardDefinition {
        name: "Felothar, Dawn of the Abzan",
        cost: cost(&[w(), b(), g()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(body()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: body(),
            },
        ],
        ..Default::default()
    }
}

/// Lotuslight Dancers — {2}{B}{G}{U} 3/6 Zombie Bard, lifelink. ETB: search your
/// library for a black, a green, and a blue card, put them into your graveyard,
/// then shuffle.
pub fn lotuslight_dancers() -> CardDefinition {
    let search = |c| Effect::Search {
        who: PlayerRef::You,
        filter: R::HasColor(c),
        to: ZoneDest::Graveyard,
    };
    CardDefinition {
        name: "Lotuslight Dancers",
        cost: cost(&[generic(2), b(), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 6,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            search(Color::Black),
            search(Color::Green),
            search(Color::Blue),
        ]))],
        ..Default::default()
    }
}

/// Eshki Dragonclaw — {1}{G}{U}{R} 4/4 Human Warrior, vigilance, trample, ward
/// {1}. At the beginning of combat on your turn, if you've cast both a creature
/// and a noncreature spell this turn, draw a card and put two +1/+1 counters on
/// Eshki.
pub fn eshki_dragonclaw() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Eshki Dragonclaw",
        cost: cost(&[generic(1), g(), u(), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Ward(WardCost::generic(1))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::All(vec![
                    Predicate::CreaturesCastThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::ONE,
                    },
                    Predicate::NoncreatureSpellsCastThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::ONE,
                    },
                ]),
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Narset, Jeskai Waymaster — {U}{R}{W} 3/4 Human Monk. At your end step, you
/// may discard your hand; if you do, draw cards equal to the number of spells
/// you've cast this turn.
pub fn narset_jeskai_waymaster() -> CardDefinition {
    CardDefinition {
        name: "Narset, Jeskai Waymaster",
        cost: cost(&[u(), r(), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::MayDo {
                description: "Discard your hand to draw cards equal to spells cast this turn?"
                    .into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::HandSizeOf(PlayerRef::You),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::SpellsCastThisTurn(PlayerRef::You),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// A 1/1 white Spirit creature token.
fn spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Revival of the Ancestors — {1}{W}{B}{G} Saga. I: three 1/1 Spirits. II:
/// distribute three +1/+1 counters among up to three creatures you control.
/// III: your creatures gain trample and lifelink until end of turn.
pub fn revival_of_the_ancestors() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Revival of the Ancestors",
        cost: cost(&[generic(1), w(), b(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    definition: spirit_token(),
                },
            ),
            (
                2,
                Effect::DistributeCounters {
                    total: Value::Const(3),
                    counter: CounterType::PlusOnePlusOne,
                    filter: R::Creature.and(R::ControlledByYou),
                    max_targets: 3,
                },
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: team(),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: team(),
                        keyword: Keyword::Lifelink,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Kishla Village — Land. Enters tapped unless you control an Island or a Swamp.
/// {T}: Add {G}. {3}{G}, {T}: Surveil 2.
pub fn kishla_village() -> CardDefinition {
    CardDefinition {
        name: "Kishla Village",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Island)
                            .or(R::HasLandType(LandType::Swamp))
                            .and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Tap { what: Selector::This }),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Green]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), g()]),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dracogenesis — {6}{R}{R} Enchantment. You may cast Dragon spells without
/// paying their mana costs.
pub fn dracogenesis() -> CardDefinition {
    CardDefinition {
        name: "Dracogenesis",
        cost: cost(&[generic(6), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You may cast Dragon spells without paying their mana costs.",
            effect: StaticEffect::CastFilteredSpellsFree {
                filter: R::HasCreatureType(CreatureType::Dragon),
            },
        }],
        ..Default::default()
    }
}

/// Death Begets Life — {5}{B}{G}{U} Sorcery. Destroy all creatures and
/// enchantments, then draw a card for each permanent destroyed this way.
/// (Draw count is the pre-destruction match count.)
pub fn death_begets_life() -> CardDefinition {
    let matching = || R::Creature.or(R::Enchantment);
    CardDefinition {
        name: "Death Begets Life",
        cost: cost(&[generic(5), b(), g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(matching())),
            },
            Effect::Destroy { what: Selector::EachPermanent(matching()) },
        ]),
        ..Default::default()
    }
}

/// Herd Heirloom — {1}{G} Artifact. {T}: Add one mana of any color, spend only
/// on a creature spell. {T}: until end of turn, target creature you control with
/// power 4+ gains trample and "when it deals combat damage to a player, draw."
pub fn herd_heirloom() -> CardDefinition {
    CardDefinition {
        name: "Herd Heirloom",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::Const(1))),
                        crate::mana::SpendRestriction::CreatureOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(
                            R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                        ),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantTriggeredAbility {
                        what: Selector::Target(0),
                        trigger: Box::new(TriggeredAbility {
                            event: EventSpec::new(
                                EventKind::DealsCombatDamageToPlayer,
                                EventScope::SelfSource,
                            ),
                            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                        }),
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Yathan Roadwatcher — {1}{W}{B}{G} 3/3 Human Scout. When it enters, if you
/// cast it, mill four cards; when you do, return a creature card with mana value
/// 3 or less from your graveyard to the battlefield.
pub fn yathan_roadwatcher() -> CardDefinition {
    CardDefinition {
        name: "Yathan Roadwatcher",
        cost: cost(&[generic(1), w(), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(4) },
                Effect::Reflexive {
                    body: Box::new(Effect::Move {
                        what: target_filtered(
                            R::Creature.and(R::InGraveyard).and(R::ManaValueAtMost(3)),
                        ),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    }),
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Great Arashin City — Land. Enters tapped unless you control a Forest or a
/// Plains. {T}: Add {B}. {1}{B}, {T}, Exile a creature card from your graveyard:
/// Create a 1/1 white Spirit creature token.
pub fn great_arashin_city() -> CardDefinition {
    CardDefinition {
        name: "Great Arashin City",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Forest)
                            .or(R::HasLandType(LandType::Plains))
                            .and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Tap { what: Selector::This }),
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
                tap_cost: true,
                mana_cost: cost(&[generic(1), b()]),
                exile_other_filter: Some((R::Creature, 1)),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: spirit_token(),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Nature's Rhythm — {X}{G}{G} Sorcery. Search your library for a creature card
/// with mana value X or less, put it onto the battlefield, then shuffle.
/// Harmonize {X}{G}{G}{G}{G}.
pub fn natures_rhythm() -> CardDefinition {
    CardDefinition {
        name: "Nature's Rhythm",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[x(), g(), g(), g(), g()]))],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::Creature.and(R::ManaValueAtMostXFromCost),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Smile at Death — {3}{W}{W} Enchantment. At the beginning of your upkeep,
/// return up to two target creature cards with power 2 or less from your
/// graveyard to the battlefield with a +1/+1 counter on each.
pub fn smile_at_death() -> CardDefinition {
    CardDefinition {
        name: "Smile at Death",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                filter: R::Creature.and(R::InYourGraveyard).and(R::PowerAtMost(2)),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Roar of Endless Song — {2}{G}{U}{R} Enchantment — Saga. I, II: create a 5/5
/// green Elephant. III: double the power and toughness of each creature you
/// control until end of turn.
pub fn roar_of_endless_song() -> CardDefinition {
    let elephant = TokenDefinition {
        name: "Elephant".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elephant], ..Default::default() },
        ..Default::default()
    };
    let make_elephant = Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: elephant };
    CardDefinition {
        name: "Roar of Endless Song",
        cost: cost(&[generic(2), g(), u(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, make_elephant.clone()),
            (2, make_elephant),
            (
                3,
                // Double each creature's P/T by adding its own current P/T.
                Effect::ForEach {
                    selector: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    body: Box::new(Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::PowerOf(Box::new(Selector::TriggerSource)),
                        toughness: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                        duration: Duration::EndOfTurn,
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Zurgo, Thunder's Decree — {R}{W}{B} 2/4 Orc Warrior with Mobilize 2.
/// (The "Warrior tokens can't be sacrificed during your end step" persistence
/// rider is approximated away — the Mobilize tokens sacrifice as normal.)
pub fn zurgo_thunders_decree() -> CardDefinition {
    CardDefinition {
        name: "Zurgo, Thunder's Decree",
        cost: cost(&[r(), w(), b()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Orc, CreatureType::Warrior], ..Default::default() },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![mobilize(2)],
        ..Default::default()
    }
}

/// Rot-Curse Rakshasa — {1}{B} 5/5 Demon with trample and decayed.
/// (Its graveyard Renew ability — exile from the graveyard to distribute decayed
/// counters — is approximated away; graveyard-activated abilities want a primitive.)
pub fn rot_curse_rakshasa() -> CardDefinition {
    CardDefinition {
        name: "Rot-Curse Rakshasa",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample, Keyword::Decayed],
        ..Default::default()
    }
}

/// Windcrag Siege — {1}{R}{W} Enchantment. As it enters, choose Mardu or Jeskai.
/// Mardu: an attack-caused trigger of a permanent you control fires an extra
/// time. Jeskai: upkeep create a 1/1 red Goblin with lifelink and haste.
pub fn windcrag_siege() -> CardDefinition {
    CardDefinition {
        name: "Windcrag Siege",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Mardu",
                static_abilities: vec![StaticAbility {
                    description: "If a creature attacking causes a triggered ability of a \
                                  permanent you control to trigger, it triggers an extra time.",
                    effect: StaticEffect::DoubleControllerAttackTriggers,
                }],
                ..Default::default()
            },
            EnterMode {
                label: "Jeskai",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(TurnStep::Upkeep),
                        EventScope::ActivePlayer,
                    ),
                    effect: Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        // lifelink/haste are printed "until end of turn"; baked on
                        // the token (negligible for a 1/1 that rarely survives).
                        definition: TokenDefinition {
                            name: "Goblin".into(),
                            power: 1,
                            toughness: 1,
                            card_types: vec![CardType::Creature],
                            colors: vec![Color::Red],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Goblin],
                                ..Default::default()
                            },
                            keywords: vec![Keyword::Lifelink, Keyword::Haste],
                            ..Default::default()
                        },
                    },
                }],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// United Battlefront — {3}{W} Sorcery. Look at the top seven cards; put up to
/// two noncreature, nonland permanent cards with mana value 3 or less onto the
/// battlefield, rest to the bottom in a random order.
pub fn united_battlefront() -> CardDefinition {
    CardDefinition {
        name: "United Battlefront",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookTopPutMatchingOntoBattlefield {
            count: Value::Const(7),
            filter: R::PermanentCard.and(R::Noncreature).and(R::Nonland).and(R::ManaValueAtMost(3)),
            then: None,
            max: Some(2),
            tapped: false,
        },
        ..Default::default()
    }
}

/// Flamehold Grappler — {U}{R}{W} 3/3 Human Monk with first strike. When it
/// enters, copy the next spell you cast this turn (you may choose new targets).
pub fn flamehold_grappler() -> CardDefinition {
    CardDefinition {
        name: "Flamehold Grappler",
        cost: cost(&[u(), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Monk], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![etb(Effect::OnYourNextSpellCastThisTurn {
            body: Box::new(Effect::CopySpellMayChooseTargets {
                what: Selector::TriggerSource,
                count: Value::ONE,
            }),
        })],
        ..Default::default()
    }
}
