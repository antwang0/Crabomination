//! Mirage (MIR), third wave — phasing shells, combat tricks and the
//! upkeep-tax creatures. Tests in `classic_sets/mir`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{on_attack, on_dies, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, StaticEffect, Value, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Sorcery], ..instant(name, c, effect) }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

/// "Whenever a player casts a [colour] spell, you may pay [cost]. If you do,
/// [body]." (Auspicious Ancestor, Purraj of Urborg.)
fn on_color_spell_may_pay(color: Color, mana: ManaCost, body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
            .with_filter(Predicate::CastSpellMatches(R::HasColor(color))),
        effect: Effect::MayPay {
            description: "Pay to keep the trigger?".into(),
            mana_cost: mana,
            body: Box::new(body),
            else_: None,
        },
    }
}

// ── Phasing shells ──────────────────────────────────────────────────────────

/// Crystal Golem — {4} 3/3 that ducks out of every end step.
pub fn crystal_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
        }],
        ..creature("Crystal Golem", cost(&[generic(4)]), vec![CreatureType::Golem], 3, 3)
    }
}

/// Dream Fighter — {2}{U} 1/1 that takes its combat partner out of phase.
pub fn dream_fighter() -> CardDefinition {
    let both_out = Effect::PhaseOut {
        what: Selector::Both(
            Box::new(Selector::This),
            Box::new(Selector::CreaturesInCombatWith(Box::new(Selector::This))),
        ),
        until_source_leaves: false,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: both_out.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: both_out,
            },
        ],
        ..creature(
            "Dream Fighter",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Teferi's Imp — {2}{U} 1/1 phasing flier that trades a card each way.
pub fn teferis_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Phasing],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PhasesOut, EventScope::SelfSource),
                effect: crate::effect::shortcut::discard(Selector::You, 1, false),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PhasesIn, EventScope::SelfSource),
                effect: crate::effect::shortcut::draw(1),
            },
        ],
        ..creature("Teferi's Imp", cost(&[generic(2), u()]), vec![CreatureType::Imp], 1, 1)
    }
}

/// Vaporous Djinn — {2}{U}{U} 3/4 flier that phases out unless you pay rent.
pub fn vaporous_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::MayPay {
                description: "Pay {U}{U} to stay in phase?".into(),
                mana_cost: cost(&[u(), u()]),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::PhaseOut {
                    what: Selector::This,
                    until_source_leaves: false,
                })),
            },
        }],
        ..creature("Vaporous Djinn", cost(&[generic(2), u(), u()]), vec![CreatureType::Djinn], 3, 4)
    }
}

/// Warping Wurm — {2}{G}{U} 1/1 phaser that grows on the way back in.
pub fn warping_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Phasing],
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::MayPay {
                    description: "Pay {2}{G}{U} to stay in phase?".into(),
                    mana_cost: cost(&[generic(2), g(), u()]),
                    body: Box::new(Effect::Noop),
                    else_: Some(Box::new(Effect::PhaseOut {
                        what: Selector::This,
                        until_source_leaves: false,
                    })),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PhasesIn, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature("Warping Wurm", cost(&[generic(2), g(), u()]), vec![CreatureType::Wurm], 1, 1)
    }
}

/// Taniwha — {3}{U}{U} 7/7 phasing trampler that drags your lands with it.
pub fn taniwha() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample, Keyword::Phasing],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::PhaseOut {
                what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                until_source_leaves: false,
            },
        }],
        ..creature("Taniwha", cost(&[generic(3), u(), u()]), vec![CreatureType::Serpent], 7, 7)
    }
}

/// Mist Dragon — {4}{U}{U} 4/4 that can toggle flight and duck out entirely.
pub fn mist_dragon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::Permanent,
                },
                ..Default::default()
            },
            ActivatedAbility {
                effect: Effect::LoseKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::Permanent,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u(), u()]),
                effect: Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
                ..Default::default()
            },
        ],
        ..creature("Mist Dragon", cost(&[generic(4), u(), u()]), vec![CreatureType::Dragon], 4, 4)
    }
}

// ── Upkeep-tax and trigger creatures ────────────────────────────────────────

/// Benthic Djinn — {2}{U}{B} 5/3 islandwalker that bleeds you dry.
pub fn benthic_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..creature(
            "Benthic Djinn",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Djinn],
            5,
            3,
        )
    }
}

/// Auspicious Ancestor — {3}{W} 2/3 that trickles life off white spells.
pub fn auspicious_ancestor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_dies(crate::effect::shortcut::gain_life(3)),
            on_color_spell_may_pay(
                Color::White,
                cost(&[generic(1)]),
                crate::effect::shortcut::gain_life(1),
            ),
        ],
        ..creature(
            "Auspicious Ancestor",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            3,
        )
    }
}

/// Purraj of Urborg — {3}{B}{B} 2/3 that grows off black spells.
pub fn purraj_of_urborg() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Has first strike as long as it's attacking.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::MatchingAmong {
                    inner: Box::new(Selector::This),
                    filter: R::IsAttacking,
                },
                keyword: Keyword::FirstStrike,
            },
        }],
        triggered_abilities: vec![on_color_spell_may_pay(
            Color::Black,
            cost(&[b()]),
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        )],
        ..creature(
            "Purraj of Urborg",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Emberwilde Caliph — {2}{U}{R} 4/4 that always swings, and always bites back.
pub fn emberwilde_caliph() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::MustAttack],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..creature(
            "Emberwilde Caliph",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Djinn],
            4,
            4,
        )
    }
}

/// Harbor Guardian — {2}{W}{U} 3/4 reach body the defender is glad to see.
pub fn harbor_guardian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![on_attack(Effect::MayDoBy {
            who: PlayerRef::DefendingPlayer,
            description: "Draw a card?".into(),
            body: Box::new(Effect::Draw {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::ONE,
            }),
        })],
        ..creature(
            "Harbor Guardian",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Gargoyle],
            3,
            4,
        )
    }
}

/// Ravenous Vampire — {3}{B}{B} 3/3 flier that eats or naps each upkeep.
pub fn ravenous_vampire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a nonartifact creature?".into(),
                filter: R::Creature.and(R::Not(Box::new(R::Artifact))),
                count: Value::ONE,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Some(Box::new(Effect::Tap { what: Selector::This })),
            },
        }],
        ..creature(
            "Ravenous Vampire",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Vampire],
            3,
            3,
        )
    }
}

/// Sabertooth Cobra — {2}{G} 2/2 that poisons whoever it connects with.
pub fn sabertooth_cobra() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddPoison {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::ONE,
            },
        }],
        ..creature("Sabertooth Cobra", cost(&[generic(2), g()]), vec![CreatureType::Snake], 2, 2)
    }
}

/// Floodgate — {3}{U} 0/5 Wall that floods the ground when it leaves.
pub fn floodgate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature
                        .and(R::Not(Box::new(R::HasColor(Color::Blue))))
                        .and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                amount: Value::HalvedRoundDown(Box::new(Value::CountOf(Box::new(
                    Selector::EachPermanent(
                        R::HasLandType(LandType::Island).and(R::ControlledByYou),
                    ),
                )))),
            },
        }],
        ..creature("Floodgate", cost(&[generic(3), u()]), vec![CreatureType::Wall], 0, 5)
    }
}

/// Spectral Guardian — {2}{W}{W} 2/3 that shields every artifact while awake.
pub fn spectral_guardian() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is untapped, noncreature artifacts have shroud.",
            effect: StaticEffect::AnthemForFilterIf {
                filter: R::Artifact.and(R::Not(Box::new(R::Creature))),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Shroud],
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::Untapped,
                },
                all_players: true,
            },
        }],
        ..creature(
            "Spectral Guardian",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Spirit],
            2,
            3,
        )
    }
}

/// Ekundu Cyclops — {3}{R} 3/4 that joins any attack.
pub fn ekundu_cyclops() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustAttackIfAnotherAttacks],
        ..creature("Ekundu Cyclops", cost(&[generic(3), r()]), vec![CreatureType::Cyclops], 3, 4)
    }
}

/// Canopy Dragon — {4}{G}{G} 4/4 trampler that can trade ground for air.
pub fn canopy_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::LoseKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Canopy Dragon",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Crimson Hellkite — {6}{R}{R}{R} 6/6 flier with an X-damage cannon.
pub fn crimson_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x()]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..creature(
            "Crimson Hellkite",
            cost(&[generic(6), r(), r(), r()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Abyssal Hunter — {3}{B} 1/1 that taps a creature and stabs it.
pub fn abyssal_hunter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Abyssal Hunter",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            1,
            1,
        )
    }
}

/// Barbed-Back Wurm — {4}{B} 4/3 that shrinks a green blocker.
pub fn barbed_back_wurm() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: target_filtered(
                    R::Creature.and(R::HasColor(Color::Green)).and(R::IsBlocking),
                ),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Barbed-Back Wurm", cost(&[generic(4), b()]), vec![CreatureType::Wurm], 4, 3)
    }
}

/// Blighted Shaman — {1}{B} 1/1 that feeds Swamps or bodies into a pump.
pub fn blighted_shaman() -> CardDefinition {
    let pump = |filter: R, n: i32| ActivatedAbility {
        tap_cost: true,
        sac_other_filter: Some((filter, 1)),
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(n),
            toughness: Value::Const(n),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            pump(R::HasLandType(LandType::Swamp), 1),
            pump(R::Creature, 2),
        ],
        ..creature(
            "Blighted Shaman",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Goblin Soothsayer — {R} 1/1 that trades a Goblin for a red team pump.
pub fn goblin_soothsayer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Goblin), 1)),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Red))),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Goblin Soothsayer",
            cost(&[r()]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Femeref Healer — {1}{W} 1/1 that shaves a point off anything.
pub fn femeref_healer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Femeref Healer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Ethereal Champion — {2}{W}{W}{W} 3/4 that soaks damage for life.
pub fn ethereal_champion() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::PreventNextDamage { target: Selector::This, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Ethereal Champion",
            cost(&[generic(2), w(), w(), w()]),
            vec![CreatureType::Avatar],
            3,
            4,
        )
    }
}

/// Urborg Panther — {2}{B} 2/2 that cashes in to kill its blocker.
pub fn urborg_panther() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::IsBlocking)),
            },
            ..Default::default()
        }],
        ..creature(
            "Urborg Panther",
            cost(&[generic(2), b()]),
            vec![CreatureType::Nightstalker, CreatureType::Cat],
            2,
            2,
        )
    }
}

/// Jungle Patrol — {3}{G} 3/2 that builds Walls it can burn for red mana.
pub fn jungle_patrol() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Wood".into(),
                        power: 0,
                        toughness: 1,
                        keywords: vec![Keyword::Defender],
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Wall],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_other_filter: Some((R::HasName("Wood".into()), 1)),
                effect: crate::effect::shortcut::add_mana(vec![Color::Red]),
                ..Default::default()
            },
        ],
        ..creature(
            "Jungle Patrol",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            2,
        )
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Alarum — {1}{W}: untap a defender and steady it.
pub fn alarum() -> CardDefinition {
    instant(
        "Alarum",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::Untap {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::IsAttacking)))),
                up_to: None,
            },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::ONE,
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Divine Retribution — {1}{W}: the whole attack's size, into one attacker.
pub fn divine_retribution() -> CardDefinition {
    instant(
        "Divine Retribution",
        cost(&[generic(1), w()]),
        Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking)),
            amount: Value::CountOf(Box::new(Selector::EachPermanent(
                R::Creature.and(R::IsAttacking),
            ))),
        },
    )
}

/// Cinder Cloud — {3}{R}{R}: removal that punishes white.
pub fn cinder_cloud() -> CardDefinition {
    instant(
        "Cinder Cloud",
        cost(&[generic(3), r(), r()]),
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::If {
                cond: Predicate::EntityMatchesAny {
                    what: Selector::Target(0),
                    filter: R::HasColor(Color::White),
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Bone Harvest — {2}{B}: stack your dead, then replace the card.
pub fn bone_harvest() -> CardDefinition {
    instant(
        "Bone Harvest",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            },
            Effect::AtNextTurnsUpkeep { body: Box::new(crate::effect::shortcut::draw(1)) },
        ]),
    )
}

/// Dazzling Beauty — {2}{W}: an unblockable attacker becomes blocked.
pub fn dazzling_beauty() -> CardDefinition {
    instant(
        "Dazzling Beauty",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::BecomeBlocked {
                what: target_filtered(R::Creature.and(R::IsAttacking).and(R::IsUnblocked)),
            },
            Effect::AtNextTurnsUpkeep { body: Box::new(crate::effect::shortcut::draw(1)) },
        ]),
    )
}

/// Final Fortune — {R}{R}: an extra turn you don't survive.
pub fn final_fortune() -> CardDefinition {
    instant(
        "Final Fortune",
        cost(&[r(), r()]),
        Effect::Seq(vec![
            Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
            Effect::AtNextEndStep {
                body: Box::new(Effect::LoseGame { who: PlayerRef::You }),
            },
        ]),
    )
}

/// Reign of Terror — {3}{B}{B}: a one-colour wrath you pay for in life.
pub fn reign_of_terror() -> CardDefinition {
    sorcery(
        "Reign of Terror",
        cost(&[generic(3), b(), b()]),
        Effect::ChooseMode(vec![
            Effect::DestroyNoRegen {
                what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Green))),
            },
            Effect::DestroyNoRegen {
                what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::White))),
            },
        ]),
    )
}

/// Reign of Chaos — {2}{R}{R}: a land and a creature of one colour.
pub fn reign_of_chaos() -> CardDefinition {
    sorcery(
        "Reign of Chaos",
        cost(&[generic(2), r(), r()]),
        Effect::ChooseMode(vec![
            Effect::Destroy {
                what: Selector::Both(
                    Box::new(target_filtered(R::HasLandType(LandType::Plains))),
                    Box::new(Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::HasColor(Color::White)),
                    }),
                ),
            },
            Effect::Destroy {
                what: Selector::Both(
                    Box::new(target_filtered(R::HasLandType(LandType::Island))),
                    Box::new(Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::HasColor(Color::Blue)),
                    }),
                ),
            },
        ]),
    )
}

/// Psychic Transfer — {4}{U}: swap life totals, if they're close enough.
pub fn psychic_transfer() -> CardDefinition {
    sorcery(
        "Psychic Transfer",
        cost(&[generic(4), u()]),
        Effect::If {
            cond: Predicate::ValueAtMost(
                Value::Max(
                    Box::new(Value::Diff(
                        Box::new(Value::LifeOf(PlayerRef::You)),
                        Box::new(Value::LifeOf(PlayerRef::Target(0))),
                    )),
                    Box::new(Value::Diff(
                        Box::new(Value::LifeOf(PlayerRef::Target(0))),
                        Box::new(Value::LifeOf(PlayerRef::You)),
                    )),
                ),
                Value::Const(5),
            ),
            then: Box::new(Effect::ExchangeLifeTotals {
                a: Selector::You,
                b: Selector::Player(PlayerRef::Target(0)),
            }),
            else_: Box::new(Effect::Noop),
        },
    )
}

/// Illumination — {W}{W}: counter an artifact or enchantment, refund its cost
/// in life.
pub fn illumination() -> CardDefinition {
    instant(
        "Illumination",
        cost(&[w(), w()]),
        Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::Artifact.or(R::Enchantment)),
                ),
            },
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::TotalManaValueOf(Box::new(Selector::Target(0))),
            },
        ]),
    )
}

/// Sirocco — {1}{R}: blue instants in hand cost 4 life to keep.
pub fn sirocco() -> CardDefinition {
    instant(
        "Sirocco",
        cost(&[generic(1), r()]),
        Effect::RevealHandDiscardMatchingUnlessPayLife {
            who: PlayerRef::Target(0),
            filter: R::HasColor(Color::Blue).and(R::HasCardType(CardType::Instant)),
            life: 4,
        },
    )
}

/// Builder's Bane — {X}{X}{R}: X artifacts die and their owners burn for them.
pub fn builders_bane() -> CardDefinition {
    sorcery(
        "Builder's Bane",
        cost(&[crate::mana::x(), crate::mana::x(), r()]),
        Effect::Seq(vec![
            // The slot ceiling is static; `CapTargetsAtX` trims it to the paid X.
            Effect::CapTargetsAtX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 6,
                    min_targets: 0,
                    filter: R::Artifact,
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ArtifactsToGraveyardFromBattlefieldThisTurn,
            },
        ]),
    )
}

// ── Enchantments & artifacts ────────────────────────────────────────────────

/// Afiya Grove — {1}{G}: three counters that walk onto your creatures.
pub fn afiya_grove() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::MoveCounters {
                    from: Selector::This,
                    to: target_filtered(R::Creature),
                    counter: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterRemoved(CounterType::PlusOnePlusOne),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::ValueAtMost(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                    Value::ZERO,
                )),
                effect: Effect::SacrificeSource,
            },
        ],
        ..enchantment("Afiya Grove", cost(&[generic(1), g()]))
    }
}

/// Chaosphere — {2}{R} world enchantment that inverts the sky.
pub fn chaosphere() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::World],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures with flying can block only creatures with flying.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::CanBlockOnlyFlying],
                    opponents: false,
                    all_players: true,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
            StaticAbility {
                description: "Creatures without flying have reach.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Reach],
                    opponents: false,
                    all_players: true,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
        ],
        ..enchantment("Chaosphere", cost(&[generic(2), r()]))
    }
}

/// Circle of Despair — {1}{W}{B}: bodies buy damage prevention.
pub fn circle_of_despair() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: Some(target_any()),
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..enchantment("Circle of Despair", cost(&[generic(1), w(), b()]))
    }
}

/// Favorable Destiny — {1}{W} Aura: bigger while white, hidden while you have
/// a friend.
pub fn favorable_destiny() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +1/+2 as long as it's white.",
                effect: StaticEffect::AnthemForFilterIf {
                    filter: R::AttachedToSource.and(R::HasColor(Color::White)),
                    power: 1,
                    toughness: 2,
                    keywords: vec![],
                    condition: Predicate::ValueAtLeast(Value::ONE, Value::ONE),
                    all_players: true,
                },
            },
            StaticAbility {
                description: "Enchanted creature has shroud as long as its controller controls another creature.",
                effect: StaticEffect::AnthemForFilterIf {
                    filter: R::AttachedToSource,
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Shroud],
                    condition: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou).and(R::Not(Box::new(
                                R::AttachedToSource,
                            ))),
                        ),
                        n: Value::ONE,
                    },
                    all_players: true,
                },
            },
        ],
        ..aura("Favorable Destiny", cost(&[generic(1), w()]), R::Creature, EquipBonus::default())
    }
}

/// Amulet of Unmaking — {5}: an expensive, unconditional exile.
pub fn amulet_of_unmaking() -> CardDefinition {
    artifact(
        "Amulet of Unmaking",
        cost(&[generic(5)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Exile {
                what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)),
            },
            ..Default::default()
        }],
    )
}

/// Barbed Foliage — {2}{G}{G}: attackers lose flanking and take a scratch.
pub fn barbed_foliage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::OpponentControl),
                effect: Effect::LoseKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Flanking,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::OpponentControl).with_filter(
                    Predicate::EntityMatchesAny {
                        what: Selector::TriggerSource,
                        filter: R::Not(Box::new(R::HasKeyword(Keyword::Flying))),
                    },
                ),
                effect: Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::ONE,
                },
            },
        ],
        ..enchantment("Barbed Foliage", cost(&[generic(2), g(), g()]))
    }
}

/// Decomposition — {1}{G} Aura that rots a black creature away.
pub fn decomposition() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has \"Cumulative upkeep—Pay 1 life.\"",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::attached_to(Selector::This),
                keyword: Keyword::CumulativeUpkeep(crate::card::CumulativeUpkeepCost::Life(1)),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::Const(2),
            },
        }],
        ..aura(
            "Decomposition",
            cost(&[generic(1), g()]),
            R::Creature.and(R::HasColor(Color::Black)),
            EquipBonus::default(),
        )
    }
}
