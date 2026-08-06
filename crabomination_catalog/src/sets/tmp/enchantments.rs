//! Tempest (TMP) enchantments and Auras. Tests in `classic_sets/tmp`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, StaticEffect, Value,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn enchantment(name: &'static str, c: ManaCost, statics: Vec<StaticAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        static_abilities: statics,
        ..Default::default()
    }
}

fn aura(name: &'static str, c: ManaCost, host: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(host) },
        ..Default::default()
    }
}

/// "At the beginning of each player's upkeep, …"
fn each_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
        effect,
    }
}

// ── Statics ─────────────────────────────────────────────────────────────────

/// Chill — {1}{U}. Red spells cost {2} more to cast, for everyone.
pub fn chill() -> CardDefinition {
    enchantment(
        "Chill",
        cost(&[generic(1), u()]),
        vec![StaticAbility {
            description: "Red spells cost {2} more to cast.",
            effect: StaticEffect::AdditionalCost { filter: R::HasColor(Color::Red), amount: 2 },
        }],
    )
}

/// Dread of Night — {B}. White creatures get -1/-1.
pub fn dread_of_night() -> CardDefinition {
    enchantment(
        "Dread of Night",
        cost(&[b()]),
        vec![StaticAbility {
            description: "White creatures get -1/-1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::White))),
                power: -1,
                toughness: -1,
            },
        }],
    )
}

/// Hanna's Custody — {2}{W}. All artifacts have shroud.
pub fn hannas_custody() -> CardDefinition {
    enchantment(
        "Hanna's Custody",
        cost(&[generic(2), w()]),
        vec![StaticAbility {
            description: "All artifacts have shroud.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact),
                keyword: Keyword::Shroud,
            },
        }],
    )
}

/// Light of Day — {3}{W}. Black creatures can't attack or block.
pub fn light_of_day() -> CardDefinition {
    let black_creatures = || Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Black)));
    enchantment(
        "Light of Day",
        cost(&[generic(3), w()]),
        vec![
            StaticAbility {
                description: "Black creatures can't attack.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: black_creatures(),
                    keyword: Keyword::CantAttack,
                },
            },
            StaticAbility {
                description: "Black creatures can't block.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: black_creatures(),
                    keyword: Keyword::CantBlock,
                },
            },
        ],
    )
}

/// Root Maze — {G}. Artifacts and lands enter tapped.
pub fn root_maze() -> CardDefinition {
    enchantment(
        "Root Maze",
        cost(&[g()]),
        vec![StaticAbility {
            description: "Artifacts and lands enter tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::EachPermanent(R::Artifact.or(R::Land)),
            },
        }],
    )
}

/// Nature's Revolt — {3}{G}{G}. All lands are 2/2 creatures that are still
/// lands.
pub fn natures_revolt() -> CardDefinition {
    enchantment(
        "Nature's Revolt",
        cost(&[generic(3), g(), g()]),
        vec![StaticAbility {
            description: "All lands are 2/2 creatures that are still lands.",
            effect: StaticEffect::MatchingLandsAreCreatures {
                filter: R::Land,
                power: 2,
                toughness: 2,
                keywords: vec![],
                creature_types: vec![],
                colors: vec![],
            },
        }],
    )
}

/// Humility — {2}{W}{W}. Every creature is a 1/1 with no abilities.
pub fn humility() -> CardDefinition {
    enchantment(
        "Humility",
        cost(&[generic(2), w(), w()]),
        vec![
            StaticAbility {
                description: "All creatures lose all abilities.",
                effect: StaticEffect::CreaturesLoseAllAbilities,
            },
            StaticAbility {
                description: "All creatures have base power and toughness 1/1.",
                effect: StaticEffect::SetBasePtForFilter {
                    applies_to: Selector::EachPermanent(R::Creature),
                    power: 1,
                    toughness: 1,
                },
            },
        ],
    )
}

// ── Upkeep / cast watchers ──────────────────────────────────────────────────

/// Ancient Runes — {2}{R}. Each upkeep, its controller's artifacts bite back.
pub fn ancient_runes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![each_upkeep(Effect::DealDamage {
            to: Selector::Player(PlayerRef::ActivePlayer),
            amount: Value::count(Selector::ControlledBy {
                who: PlayerRef::ActivePlayer,
                filter: R::Artifact,
            }),
        })],
        ..enchantment("Ancient Runes", cost(&[generic(2), r()]), vec![])
    }
}

/// Eladamri's Vineyard — {G}. Every player's first main phase starts with
/// {G}{G} in the pool.
pub fn eladamris_vineyard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::AnyPlayer,
            ),
            effect: Effect::AddMana {
                who: PlayerRef::ActivePlayer,
                pool: ManaPayload::Colors(vec![Color::Green, Color::Green]),
            },
        }],
        ..enchantment("Eladamri's Vineyard", cost(&[g()]), vec![])
    }
}

/// Havoc — {1}{R}. An opponent's white spell costs them 2 life.
pub fn havoc() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::White),
                },
            ),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
            },
        }],
        ..enchantment("Havoc", cost(&[generic(1), r()]), vec![])
    }
}

/// Insight — {2}{U}. An opponent's green spell draws you a card.
pub fn insight() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Green),
                },
            ),
            effect: draw(1),
        }],
        ..enchantment("Insight", cost(&[generic(2), u()]), vec![])
    }
}

/// Mirri's Guile — {G}. Sort the top three each upkeep.
pub fn mirris_guile() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Look at the top three cards and reorder them".to_string(),
                body: Box::new(Effect::RearrangeTop { who: PlayerRef::You, amount: Value::Const(3) }),
            },
        }],
        ..enchantment("Mirri's Guile", cost(&[g()]), vec![])
    }
}

/// Orim's Prayer — {1}{W}{W}. Each attacker aimed at you is a point of life.
pub fn orims_prayer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..enchantment("Orim's Prayer", cost(&[generic(1), w(), w()]), vec![])
    }
}

/// Field of Souls — {2}{W}{W}. Every nontoken creature of yours that dies
/// leaves a Spirit behind.
pub fn field_of_souls() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                definition: TokenDefinition {
                    name: "Spirit".to_string(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Spirit],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
                count: Value::ONE,
            },
        }],
        ..enchantment("Field of Souls", cost(&[generic(2), w(), w()]), vec![])
    }
}

/// Death Pits of Rath — {3}{B}{B}. Any damage is lethal.
pub fn death_pits_of_rath() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::AnyPlayer),
            effect: Effect::DestroyNoRegen { what: Selector::TriggerSource },
        }],
        ..enchantment("Death Pits of Rath", cost(&[generic(3), b(), b()]), vec![])
    }
}

// ── Activated enchantments ──────────────────────────────────────────────────

/// Dauthi Embrace — {2}{B}. {B}{B}: hand out shadow.
pub fn dauthi_embrace() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Shadow,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment("Dauthi Embrace", cost(&[generic(2), b()]), vec![])
    }
}

/// Fevered Convulsions — {B}{B}. {2}{B}{B}: a -1/-1 counter.
pub fn fevered_convulsions() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), b()]),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Fevered Convulsions", cost(&[b(), b()]), vec![])
    }
}

/// Gerrard's Battle Cry — {W}. {2}{W}: a team-wide +1/+1.
pub fn gerrards_battle_cry() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::PumpPT {
                what: crate::effect::shortcut::each_your_creature(),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment("Gerrard's Battle Cry", cost(&[w()]), vec![])
    }
}

/// Pegasus Refuge — {3}{W}. {2}, Discard a card: a flying Pegasus.
pub fn pegasus_refuge() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                definition: TokenDefinition {
                    name: "Pegasus".to_string(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Pegasus],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Pegasus Refuge", cost(&[generic(3), w()]), vec![])
    }
}

/// Broken Fall — {2}{G}. Bounce it to regenerate something.
pub fn broken_fall() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            return_permanent_cost: Some(R::IsSource),
            effect: Effect::Regenerate { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..enchantment("Broken Fall", cost(&[generic(2), g()]), vec![])
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Hero's Resolve — {1}{W} Aura. Enchanted creature gets +1/+5.
pub fn heros_resolve() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +1/+5.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: 1,
                toughness: 5,
            },
        }],
        ..aura("Hero's Resolve", cost(&[generic(1), w()]), R::Creature)
    }
}

/// Frog Tongue — {G} Aura. Cantrips, then grants reach.
pub fn frog_tongue() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has reach.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Reach,
            },
        }],
        ..aura("Frog Tongue", cost(&[g()]), R::Creature)
    }
}

/// Crown of Flames — {R} Aura. {R}: +1/+0; {R}: bounce it.
pub fn crown_of_flames() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::PumpPT {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::Move {
                    what: Selector::This,
                    to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            },
        ],
        ..aura("Crown of Flames", cost(&[r()]), R::Creature)
    }
}

/// Endless Scream — {X}{B} Aura. Enters with X scream counters and pumps power
/// by that many.
pub fn endless_scream() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Scream, Value::XFromCost)),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +1/+0 for each scream counter on this Aura.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Scream,
                },
                toughness: Value::ZERO,
            },
        }],
        ..aura("Endless Scream", cost(&[crate::mana::x(), b()]), R::Creature)
    }
}

/// Sadistic Glee — {B} Aura. Every death feeds the enchanted creature.
pub fn sadistic_glee() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::AddCounter {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..aura("Sadistic Glee", cost(&[b()]), R::Creature)
    }
}

// ── Batch: the Rathi enchantment tail ───────────────────────────────────────

/// Warmth — {1}{W}. An opponent's red spell pays you 2 life.
pub fn warmth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Red),
                },
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..enchantment("Warmth", cost(&[generic(1), w()]), vec![])
    }
}

/// Sarcomancy — {B}. A free 2/2 Zombie that pings you once the Zombies are all
/// gone.
pub fn sarcomancy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Zombie".to_string(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Zombie],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::HasCreatureType(CreatureType::Zombie)),
                    n: Value::ONE,
                }))),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::You),
                    amount: Value::ONE,
                },
            },
        ],
        ..enchantment("Sarcomancy", cost(&[b()]), vec![])
    }
}

/// Storm Front — {G}. {G}{G}: tap a flier.
pub fn storm_front() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            },
            ..Default::default()
        }],
        ..enchantment("Storm Front", cost(&[g()]), vec![])
    }
}

/// Tooth and Claw — {3}{R}. Grind two creatures into a 3/1 Carnivore.
pub fn tooth_and_claw() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 2)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Carnivore".to_string(),
                    power: 3,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Beast],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..enchantment("Tooth and Claw", cost(&[generic(3), r()]), vec![])
    }
}

/// Shimmering Wings — {U} Aura. Flying, and a {U} rebuy.
pub fn shimmering_wings() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Flying,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Move {
                what: Selector::This,
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..aura("Shimmering Wings", cost(&[u()]), R::Creature)
    }
}

/// Spinal Graft — {1}{B} Aura. +3/+3, but the host dies the moment anything
/// targets it.
pub fn spinal_graft() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +3/+3.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: 3,
                toughness: 3,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::EnchantedBySource),
            effect: Effect::DestroyNoRegen { what: Selector::TriggerSource },
        }],
        ..aura("Spinal Graft", cost(&[generic(1), b()]), R::Creature)
    }
}

/// Steal Enchantment — {U}{U} Aura on an enchantment: you control it.
pub fn steal_enchantment() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        ..aura("Steal Enchantment", cost(&[u(), u()]), R::Enchantment)
    }
}

/// Recycle — {4}{G}{G}. No draw step, but every card you play replaces itself;
/// your hand caps at two (CR 613.11 — the newest cap wins).
pub fn recycle() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Skip your draw step.",
                effect: StaticEffect::ControllerSkipsDrawStep,
            },
            StaticAbility {
                description: "Your maximum hand size is two.",
                effect: StaticEffect::ControllerMaxHandSize(2),
            },
        ],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                effect: draw(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: draw(1),
            },
        ],
        ..enchantment("Recycle", cost(&[generic(4), g(), g()]), vec![])
    }
}
