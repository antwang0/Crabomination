//! Commander: three-card gaps of EDHREC's commanders ranked 4501-5500 (the
//! 2026-10-14 scan). Tests in `tests/recent_b/cmdr_rank5k_c.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, ConditionalEquipBonus, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    MayPlayDuration, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, r, u, w};
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
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn your_step(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

fn make(def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(def) }
}

fn aura(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..Default::default()
    }
}

fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// "Whenever a source you control deals damage to you" — a damaged-you event
/// whose dealer (the trigger source) is yours.
fn your_source_damages_you() -> EventSpec {
    EventSpec::new(EventKind::PlayerDamaged, EventScope::YourControl)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ControlledByYou })
}

fn doombot() -> TokenDefinition {
    TokenDefinition {
        name: "Doombot".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot, CreatureType::Villain], ..Default::default() },
        ..Default::default()
    }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

/// "{2}: Add one mana of any color."
fn two_for_any() -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
        ..Default::default()
    }
}

/// Go-Shintai of Hidden Cruelty — {3}{B} legendary 2/2 deathtouch Shrine
/// enchantment creature. At the beginning of your end step, you may pay {1};
/// when you do, destroy target creature with toughness X or less, X the
/// Shrines you control.
pub fn go_shintai_of_hidden_cruelty() -> CardDefinition {
    let shrines = Value::count(Selector::EachPermanent(
        R::HasEnchantmentSubtype(EnchantmentSubtype::Shrine).and(R::ControlledByYou),
    ));
    CardDefinition {
        name: "Go-Shintai of Hidden Cruelty",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Shrine], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} to destroy a creature with toughness up to your Shrines?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::WithX {
                    x: shrines,
                    body: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::Destroy {
                            what: target_filtered(R::Creature.and(R::ToughnessAtMostXFromCost)),
                        }),
                    }),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Goblin Artillery — {1}{R}{R} 1/3 Goblin Warrior. {T}: 2 damage to any target
/// and 3 damage to you.
pub fn goblin_artillery() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(2) },
                Effect::DealDamage { to: Selector::You, amount: Value::Const(3) },
            ]),
            ..Default::default()
        }],
        ..creature("Goblin Artillery", cost(&[generic(1), r(), r()]), vec![CreatureType::Goblin, CreatureType::Warrior], 1, 3)
    }
}

/// Energy Refractor — {2} artifact. It enters drawing a card. {2}: add one
/// mana of any color.
pub fn energy_refractor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        activated_abilities: vec![two_for_any()],
        ..artifact("Energy Refractor", cost(&[generic(2)]))
    }
}

/// Urn of Godfire — {1} artifact. {2}: add one mana of any color. {6}, {T},
/// sacrifice it: destroy target creature or enchantment.
pub fn urn_of_godfire() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            two_for_any(),
            ActivatedAbility {
                mana_cost: cost(&[generic(6)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Destroy { what: target_filtered(R::Creature.or(R::Enchantment)) },
                ..Default::default()
            },
        ],
        ..artifact("Urn of Godfire", cost(&[generic(1)]))
    }
}

/// Guerrilla Gorilla — {1}{G} 2/2 Ape Soldier Hero, reach. Sacrifice it:
/// destroy target noncreature artifact or noncreature enchantment, as a
/// sorcery.
pub fn guerrilla_gorilla() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment).and(R::Not(Box::new(R::Creature)))),
            },
            ..Default::default()
        }],
        ..creature(
            "Guerrilla Gorilla",
            cost(&[generic(1), g()]),
            vec![CreatureType::Ape, CreatureType::Soldier, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Photon, Lady of Light — {3}{W} legendary 3/3 flying Human Hero. Whenever
/// she attacks, flicker up to one other target creature you control.
pub fn photon_lady_of_light() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::ExileAndReturnToOwner {
                what: target_filtered(yours(R::Creature).and(R::OtherThanSource)),
            }),
        })],
        ..legend("Photon, Lady of Light", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3)
    }
}

/// Prophesied End — {1}{W} instant. Destroy target creature; if it wasn't
/// attacking, its controller draws a card.
pub fn prophesied_end() -> CardDefinition {
    CardDefinition {
        name: "Prophesied End",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        // Read before the destroy: whether it was attacking is its last state.
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: target_filtered(R::Creature),
                    filter: R::IsAttacking,
                })),
                then: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Destroy { what: Selector::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Jennika, Bad Apple Big Sister — {4}{W} legendary 3/3 Mutant Ninja Turtle.
/// She enters with a 2/2 red Mutant token. Plainscycling {2}.
pub fn jennika_bad_apple_big_sister() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains)],
        triggered_abilities: vec![etb(make(TokenDefinition {
            name: "Mutant".into(),
            power: 2,
            toughness: 2,
            colors: vec![Color::Red],
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Mutant], ..Default::default() },
            ..Default::default()
        }))],
        ..legend(
            "Jennika, Bad Apple Big Sister",
            cost(&[generic(4), w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            3,
            3,
        )
    }
}

/// Settle Beyond Reality — {4}{W} sorcery. Choose one or both — exile target
/// creature you don't control; flicker target creature you control.
pub fn settle_beyond_reality() -> CardDefinition {
    CardDefinition {
        name: "Settle Beyond Reality",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
                    to: ZoneDest::Exile,
                },
                Effect::ExileAndReturnToOwner { what: target_filtered(yours(R::Creature)) },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Namor, Scourge of the Seas — {3}{U}{U} legendary 4/4 flying Mutant Merfolk
/// Villain. Each other Merfolk you control with a +1/+1 counter has flying.
/// At the beginning of combat on your turn, target creature you control
/// connives.
pub fn namor_scourge_of_the_seas() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab(
            "Each other Merfolk you control with a +1/+1 counter on it has flying.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    yours(R::HasCreatureType(CreatureType::Merfolk))
                        .and(R::WithCounter(CounterType::PlusOnePlusOne))
                        .and(R::OtherThanSource),
                ),
                keyword: Keyword::Flying,
            },
        )],
        triggered_abilities: vec![your_step(
            TurnStep::BeginCombat,
            Effect::Connive { what: target_filtered(yours(R::Creature)), amount: Value::ONE },
        )],
        ..legend(
            "Namor, Scourge of the Seas",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Mutant, CreatureType::Merfolk, CreatureType::Villain],
            4,
            4,
        )
    }
}

/// Atlantean Cavalry — {2}{U} 3/2 vigilance Merfolk Soldier. Whenever you draw
/// your second card each turn, a +1/+1 counter on it.
pub fn atlantean_cavalry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Atlantean Cavalry", cost(&[generic(2), u()]), vec![CreatureType::Merfolk, CreatureType::Soldier], 3, 2)
    }
}

/// Atlantean Skirmisher — {1}{U} 1/2 Merfolk Rogue. Whenever it attacks, it
/// connives.
pub fn atlantean_skirmisher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Connive { what: Selector::This, amount: Value::ONE })],
        ..creature("Atlantean Skirmisher", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Rogue], 1, 2)
    }
}

/// Iron Man, Modern Marvel — {2}{U}{U} legendary 3/3 flying Human Hero
/// artifact creature. Other artifact creatures you control get +1/+1.
/// Whenever he attacks, if you control another artifact creature, draw a card.
pub fn iron_man_modern_marvel() -> CardDefinition {
    let other_artifact_creatures = || yours(R::Artifact.and(R::Creature)).and(R::OtherThanSource);
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab(
            "Other artifact creatures you control get +1/+1.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(other_artifact_creatures()),
                power: 1,
                toughness: 1,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::SelectorExists(Selector::EachPermanent(other_artifact_creatures()))),
            effect: draw(1),
        }],
        ..legend("Iron Man, Modern Marvel", cost(&[generic(2), u(), u()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3)
    }
}

/// Hydraulic Helper — {1}{U} 2/3 defender Robot artifact creature. {T}: add
/// {U}, not spendable on a nonartifact spell.
pub fn hydraulic_helper() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Blue])),
                    SpendRestriction::NoNonartifactSpells,
                ),
            },
            ..Default::default()
        }],
        ..creature("Hydraulic Helper", cost(&[generic(1), u()]), vec![CreatureType::Robot], 2, 3)
    }
}

/// Iron Man, Futurist Paragon — {4}{U}{U} legendary 5/5 flying Human Hero
/// artifact creature. At the beginning of combat on your turn, target artifact
/// or creature becomes an artifact creature with base 5/5 and gains flying.
pub fn iron_man_futurist_paragon() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![your_step(
            TurnStep::BeginCombat,
            Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: target_filtered(R::Artifact.or(R::Creature)),
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    creature_types: vec![],
                    keywords: vec![Keyword::Flying],
                    duration: Duration::Permanent,
                },
                Effect::AddCardTypeIndefinitely { what: Selector::Target(0), card_type: CardType::Artifact, until_eot: false },
            ]),
        )],
        ..legend(
            "Iron Man, Futurist Paragon",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Human, CreatureType::Hero],
            5,
            5,
        )
    }
}

/// Splinter, Aging Champion — {2}{W} legendary 2/2 Mutant Ninja Rat. When he
/// enters, destroy up to one target tapped creature. When he leaves, you and
/// another target player each draw a card.
pub fn splinter_aging_champion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) }),
            }),
            // "Another target player": in a free-for-all every other player
            // is an opponent.
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    draw(1),
                    Effect::Draw { who: target_filtered(R::OpponentPlayer), amount: Value::ONE },
                ]),
            },
        ],
        ..legend(
            "Splinter, Aging Champion",
            cost(&[generic(2), w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Rat],
            2,
            2,
        )
    }
}

/// Pegasus Guardian // Rescue the Foal — {5}{W} 3/3 flying Pegasus. At the
/// beginning of your end step, if a permanent you controlled left the
/// battlefield this turn, a 1/1 white flying Pegasus. Adventure {1}{W}
/// instant: flicker target creature you control.
pub fn pegasus_guardian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
            effect: make(TokenDefinition {
                name: "Pegasus".into(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes { creature_types: vec![CreatureType::Pegasus], ..Default::default() },
                keywords: vec![Keyword::Flying],
                ..Default::default()
            }),
        }],
        adventure: Some(Box::new(Adventure {
            name: "Rescue the Foal",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::ExileAndReturnToOwner { what: target_filtered(yours(R::Creature)) },
        })),
        ..creature("Pegasus Guardian", cost(&[generic(5), w()]), vec![CreatureType::Pegasus], 3, 3)
    }
}

/// Set Phasers to . . . — {W} instant. Choose one — tap target creature and a
/// stun counter on it; or 3 damage to target attacking creature.
pub fn set_phasers_to() -> CardDefinition {
    CardDefinition {
        name: "Set Phasers to . . .",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
            ]),
            Effect::DealDamage { to: target_filtered(R::Creature.and(R::IsAttacking)), amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Casting of Bones — {2}{B} Aura. When the enchanted creature dies, draw
/// three cards, then discard one of them.
pub fn casting_of_bones() -> CardDefinition {
    CardDefinition {
        // "Discard one of them" is a plain discard of one: the pick isn't
        // restricted to the three drawn (INCOMPLETE_CARDS).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                draw(3),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        ..aura("Casting of Bones", cost(&[generic(2), b()]))
    }
}

/// Necrosynthesis — {1}{B} Aura. The enchanted creature has "whenever another
/// creature dies, a +1/+1 counter on this". When it dies, look at the top X
/// (its power), one to hand, the rest on the bottom at random.
pub fn necrosynthesis() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource }),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            }],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::TriggerSource)),
                rest_bottom_random: true,
                ..Default::default()
            })),
        }],
        ..aura("Necrosynthesis", cost(&[generic(1), b()]))
    }
}

/// Stick, Fearless Mentor — {1}{R} legendary 2/1 first strike Human Monk
/// Hero. Once each turn, whenever a source you control deals damage to you,
/// impulse the top card until the end of your next turn.
pub fn stick_fearless_mentor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: your_source_damages_you().once_per_turn(),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..legend(
            "Stick, Fearless Mentor",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Monk, CreatureType::Hero],
            2,
            1,
        )
    }
}

/// Daredevil, Fearless Fighter — {2}{R} legendary 3/3 Human Hero. Whenever a
/// source you control deals damage to you, he deals that much to target
/// opponent. Whenever he attacks, exile the top card, he deals you its mana
/// value, and you may play it this turn.
pub fn daredevil_fearless_fighter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_source_damages_you(),
                effect: Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::TriggerEventAmount },
            },
            on_attack(Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                Effect::DealDamage { to: Selector::You, amount: Value::ManaValueOf(Box::new(Selector::LastMoved)) },
            ])),
        ],
        ..legend("Daredevil, Fearless Fighter", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3)
    }
}

/// Hellcat, Undying Vigilante — {G}{G} legendary 2/2 haste Human Hero. When
/// she dies, she returns with a +1/+1 counter, loses all abilities and gains
/// haste.
pub fn hellcat_undying_vigilante() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
            },
            Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::LoseAllAbilities { what: Selector::LastMoved, duration: Duration::Permanent },
            Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::Permanent },
        ]))],
        ..legend("Hellcat, Undying Vigilante", cost(&[g(), g()]), vec![CreatureType::Human, CreatureType::Hero], 2, 2)
    }
}

/// Medic's Kitesail — {2} Equipment. Equipped creature gets +1/+0 and has
/// flying and "whenever this attacks, you gain 1 life". Equip {2}.
pub fn medics_kitesail() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 0,
            keywords: vec![Keyword::Flying],
            triggered_abilities: vec![on_attack(Effect::GainLife { who: Selector::You, amount: Value::ONE })],
            ..Default::default()
        }),
        ..artifact("Medic's Kitesail", cost(&[generic(2)]))
    }
}

/// Castle Doom — land. {T}: {C}. {T}: one mana of any color for artifact
/// spells. {3}, {T}, sacrifice an artifact: a 3/3 Doombot Robot Villain
/// artifact creature token, as a sorcery.
pub fn castle_doom() -> CardDefinition {
    CardDefinition {
        name: "Castle Doom",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            // `ArtifactOnly` also admits artifact abilities (INCOMPLETE_CARDS).
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::ArtifactOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_other_filter: Some((R::Artifact, 1)),
                sorcery_speed: true,
                effect: make(doombot()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Doctor Doom — {4}{B}{B} legendary 3/3 Human Scientist Villain. He enters
/// with two 3/3 Doombot tokens; indestructible while you control an artifact
/// creature or a Plan; at your end step you draw a card and lose 1 life.
pub fn doctor_doom() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![make(doombot()), make(doombot())])),
            your_step(
                TurnStep::End,
                Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]),
            ),
        ],
        static_abilities: vec![static_ab(
            "As long as you control an artifact creature or a Plan, Doctor Doom has indestructible.",
            StaticEffect::WhileCondition {
                condition: Predicate::SelectorExists(Selector::EachPermanent(yours(
                    R::Artifact.and(R::Creature).or(R::HasEnchantmentSubtype(EnchantmentSubtype::Plan)),
                ))),
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Indestructible }),
            },
        )],
        ..legend(
            "Doctor Doom",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Invoke Despair — {1}{B}{B}{B}{B} sorcery. Target opponent sacrifices a
/// creature; if they can't, they lose 2 life and you draw. Then the same for
/// an enchantment and a planeswalker.
pub fn invoke_despair() -> CardDefinition {
    // The first step declares the opponent (slot 0); the later two reuse it.
    let step = |who: Selector, filter: R| Effect::If {
        cond: Predicate::ValueAtLeast(Value::PermanentCountControlledByMatching(PlayerRef::Target(0), filter.clone()), Value::ONE),
        then: Box::new(Effect::Sacrifice { who: who.clone(), count: Value::ONE, filter }),
        else_: Box::new(Effect::Seq(vec![Effect::LoseLife { who, amount: Value::Const(2) }, draw(1)])),
    };
    CardDefinition {
        name: "Invoke Despair",
        cost: cost(&[generic(1), b(), b(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            step(target_filtered(R::OpponentPlayer), R::Creature),
            step(Selector::Target(0), R::Enchantment),
            step(Selector::Target(0), R::Planeswalker),
        ]),
        ..Default::default()
    }
}

/// Huang Zhong, Shu General — {2}{W}{W} legendary 2/3 Human Soldier; it can't
/// be blocked by more than one creature.
pub fn huang_zhong_shu_general() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        ..legend("Huang Zhong, Shu General", cost(&[generic(2), w(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 3)
    }
}

/// Harvest Hand // Scrounged Scythe — {3} 2/2 Scarecrow artifact creature;
/// when it dies, it returns transformed: an Equipment giving +1/+1, and
/// menace to a Human. Equip {2}.
pub fn harvest_hand() -> CardDefinition {
    let scythe = CardDefinition {
        name: "Scrounged Scythe",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::HasCreatureType(CreatureType::Human),
                keywords: vec![Keyword::Menace],
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_dies(Effect::ExileSelfReturnTransformed)],
        back_face: Some(Box::new(scythe)),
        ..creature("Harvest Hand", cost(&[generic(3)]), vec![CreatureType::Scarecrow], 2, 2)
    }
}

/// Bonds of Faith — {1}{W} Aura. A Human gets +2/+2; anything else can't
/// attack or block.
pub fn bonds_of_faith() -> CardDefinition {
    let human = || R::HasCreatureType(CreatureType::Human);
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            conditional: vec![
                ConditionalEquipBonus { host_filter: human(), power: 2, toughness: 2, ..Default::default() },
                ConditionalEquipBonus {
                    host_filter: R::Not(Box::new(human())),
                    keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
        ..aura("Bonds of Faith", cost(&[generic(1), w()]))
    }
}

/// Pipsqueak, Rebel Strongarm — {2}{W} legendary 4/4 Human Rebel Ally; he
/// can't attack alone unless he has a +1/+1 counter.
pub fn pipsqueak_rebel_strongarm() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Pipsqueak can't attack alone unless he has a +1/+1 counter on him.",
            StaticEffect::WhileCondition {
                condition: Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::WithCounter(CounterType::PlusOnePlusOne),
                })),
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::CantAttackAlone }),
            },
        )],
        ..legend(
            "Pipsqueak, Rebel Strongarm",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            4,
            4,
        )
    }
}

/// Symmetry Matrix — {4} artifact. Whenever a creature you control with power
/// equal to its toughness enters, you may pay {1} to draw a card.
pub fn symmetry_matrix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                Predicate::ValueEquals(
                    Value::PowerOf(Box::new(Selector::TriggerSource)),
                    Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                ),
            ])),
            effect: Effect::MayPay {
                description: "Pay {1} to draw a card?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(draw(1)),
                else_: None,
            },
        }],
        ..artifact("Symmetry Matrix", cost(&[generic(4)]))
    }
}

/// Gleam of Authority — {1}{W} Aura. +1/+1 per +1/+1 counter on other
/// creatures you control; vigilance and "{W}, {T}: Bolster 1".
pub fn gleam_of_authority() -> CardDefinition {
    let counters = || Value::CountersOn {
        what: Box::new(Selector::EachPermanent(yours(R::Creature).and(R::Not(Box::new(R::IsHostOfSource))))),
        kind: CounterType::PlusOnePlusOne,
    };
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "Enchanted creature gets +1/+1 for each +1/+1 counter on other creatures you control.",
                StaticEffect::PumpPTByValue { applies_to: host(), power: counters(), toughness: counters() },
            ),
            static_ab(
                "Enchanted creature has vigilance.",
                StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::Vigilance },
            ),
        ],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                effect: crate::effect::shortcut::bolster(1),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Gleam of Authority", cost(&[generic(1), w()]))
    }
}

/// Spider-Man, Miles Morales — {4}{G}{G} legendary 5/5 vigilance, trample
/// Spider Human Hero. Whenever he enters or attacks, a +1/+1 counter on each
/// other creature you control, and they gain trample until end of turn.
pub fn spider_man_miles_morales() -> CardDefinition {
    let pump = || {
        let others = || Selector::EachPermanent(yours(R::Creature).and(R::OtherThanSource));
        Effect::Seq(vec![
            Effect::AddCounter { what: others(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::GrantKeyword { what: others(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![etb(pump()), on_attack(pump())],
        ..legend(
            "Spider-Man, Miles Morales",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            5,
            5,
        )
    }
}
