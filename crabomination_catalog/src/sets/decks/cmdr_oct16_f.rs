//! Commander: the 2026-10-16 (`0128tsCy`) rescan's gaps — Winter Soldier, Luke
//! Cage, Captain America Liberator, Doorman, The Prismatic Piper; Pia Nalaar,
//! Kraven, Ghost, Boomerang; Rasaad yn Bashir's decks, Faceless One // Criminal
//! Past. Tests in `tests/recent_b/cmdr_oct16_f.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, StateTriggeredAbility, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{cast_is_noncreature, etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, hybrid, r, w};
use crabomination_base::tokens::treasure_token;
use std::sync::Arc;

/// Blur of Heroism — {2}{W} Enchantment. ETB: draw a card. You may cast Hero
/// spells as though they had flash.
pub fn blur_of_heroism() -> CardDefinition {
    CardDefinition {
        name: "Blur of Heroism",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        static_abilities: vec![StaticAbility {
            description: "You may cast Hero spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: R::HasCreatureType(CreatureType::Hero) },
        }],
        ..Default::default()
    }
}

/// Death to Our Enemies — {2}{R} Enchantment — Plan. Each noncreature spell
/// you cast makes a tapped Treasure and a plan counter; the fourth sacrifices
/// it, and when you do it deals 7 damage divided among one or two targets.
pub fn death_to_our_enemies() -> CardDefinition {
    CardDefinition {
        name: "Death to Our Enemies",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Plan], ..Default::default() },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(cast_is_noncreature()),
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(TokenDefinition { tapped: true, ..treasure_token() }),
                    },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Plan, amount: Value::ONE },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::Plan), EventScope::SelfSource)
                    .with_filter(Predicate::SourceHasCountersAtLeast { counter: CounterType::Plan, n: 4 }),
                effect: Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(Effect::ReflexiveTrigger {
                            body: Box::new(Effect::DealDamageDivided {
                                retaliate_to_source: false,
                                total: Value::Const(7),
                                filter: R::any_target(),
                                max_targets: 2,
                            }),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Origin of Captain America — {2}{W} Saga. I: a +1/+1 counter on target
/// creature you control, first strike and vigilance this turn. II: a Sturdy
/// Shield Equipment token (+1/+2, equip {2}). III: tap up to one target
/// creature and put a stun counter on it.
pub fn origin_of_captain_america() -> CardDefinition {
    let shield = TokenDefinition {
        name: "Sturdy Shield".into(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 2, ..Default::default() }),
        ..Default::default()
    };
    let yours = || target_filtered(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Origin of Captain America",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::AddCounter { what: yours(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                ]),
            ),
            (2, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(shield) }),
            (
                3,
                Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 1,
                    filter: R::Creature,
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: Selector::Target(0) },
                        Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                    ])),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Fishing Gear — {3} Equipment, equip {2}. Equipped creature's combat damage
/// to a player exiles that player's top card: a permanent card you may put onto
/// the battlefield under your control; otherwise a 1/1 blue Fish.
pub fn fishing_gear() -> CardDefinition {
    let fish = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Fish".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            colors: vec![Color::Blue],
            subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
            ..Default::default()
        }),
    };
    let permanent = || R::Permanent;
    let trigger = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::ExileTopOfLibrary {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::ONE,
                link_to_source: false,
                face_down: false,
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::ExiledThisResolution { filter: permanent() }),
                then: Box::new(Effect::MayDoElse {
                    description: "Put the exiled permanent card onto the battlefield under your control?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::ExiledThisResolution { filter: permanent() },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Box::new(fish()),
                }),
                else_: Box::new(fish()),
            },
        ]),
    };
    CardDefinition {
        name: "Fishing Gear",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { triggered_abilities: vec![trigger], ..Default::default() }),
        ..Default::default()
    }
}

/// Sensation Gorger — {1}{R}{R} 2/2 Goblin Shaman. Kinship: at your upkeep, if
/// the top card of your library shares a creature type with it, you may reveal
/// it; if you do, each player discards their hand, then draws four cards.
pub fn sensation_gorger() -> CardDefinition {
    CardDefinition {
        name: "Sensation Gorger",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Shaman], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                    filter: R::SharesCreatureTypeWithSource,
                },
                then: Box::new(Effect::MayDo {
                    description: "Reveal the top card to have each player discard their hand and draw four?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RevealTopOfLibrary { who: PlayerRef::You },
                        Effect::Discard { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(100), random: false },
                        Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(4) },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

// ── batch 2: Pia Nalaar, Kraven Proud Predator, Ghost, Boomerang ────────────

fn thopter(name: &str, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        ..Default::default()
    }
}

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

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

/// Ghirapur Gearcrafter — {2}{R} 2/1 Human Artificer. ETB: a 1/1 flying Thopter.
pub fn ghirapur_gearcrafter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(thopter("Thopter", 1, 1)),
        })],
        ..creature("Ghirapur Gearcrafter", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 1)
    }
}

/// Yotia Declares War — {1}{R} read-ahead Saga. I: a 0/2 flying Thopter named
/// Ornithopter. II: tap any number of your untapped artifacts; that much damage
/// to target creature or planeswalker. III: up to one target artifact you
/// control is a 4/4 artifact creature until end of turn.
pub fn yotia_declares_war() -> CardDefinition {
    CardDefinition {
        name: "Yotia Declares War",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        read_ahead: true,
        saga_chapters: vec![
            (1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(thopter("Ornithopter", 0, 2)) }),
            (
                2,
                // The target is picked as the chapter triggers rather than on
                // the reflexive "when you do"; the damage is the tapped count.
                Effect::TapAnyNumberThen {
                    filter: R::Artifact,
                    body: Box::new(Effect::DealDamage {
                        to: target_filtered(R::Creature.or(R::Planeswalker)),
                        amount: Value::XFromCost,
                    }),
                },
            ),
            (
                3,
                Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 1,
                    filter: R::Artifact.and(R::ControlledByYou),
                    effect: Box::new(Effect::BecomeCreature {
                        what: Selector::Target(0),
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        creature_types: vec![],
                        keywords: vec![],
                        duration: Duration::EndOfTurn,
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Welding Sparks — {2}{R} instant: 3 plus your artifact count damage to target
/// creature.
pub fn welding_sparks() -> CardDefinition {
    CardDefinition {
        name: "Welding Sparks",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::Sum(vec![Value::Const(3), Value::count(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)))]),
        },
        ..Default::default()
    }
}

/// Savage Mansion — R/G tapland; {4}, {T}: surveil 1.
pub fn savage_mansion() -> CardDefinition {
    let mut d = crate::sets::dual_land_untyped("Savage Mansion", Color::Red, Color::Green, vec![]);
    d.static_abilities.push(crate::sets::enters_tapped());
    d.activated_abilities.push(ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(4)]),
        effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        ..Default::default()
    });
    d
}

/// Decimator of the Provinces — {10} 7/7 Eldrazi Boar, trample, haste; emerge
/// {6}{G}{G}{G}. Cast: creatures you control get +2/+2 and trample this turn.
pub fn decimator_of_the_provinces() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        alternative_cost: Some(crate::effect::shortcut::emerge(cost(&[generic(6), g(), g(), g()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: team(), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: team(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Decimator of the Provinces", cost(&[generic(10)]), vec![CreatureType::Eldrazi, CreatureType::Boar], 7, 7)
    }
}

/// Kraven's Last Hunt — {3}{G} Saga. I: mill five; when you do, damage equal to
/// the greatest power among creature cards in your graveyard to target
/// creature. II: target creature you control +2/+2. III: return target creature
/// card from your graveyard to your hand.
pub fn kravens_last_hunt() -> CardDefinition {
    let greatest = Value::GreatestPowerAmongCards(Box::new(Selector::CardsInZone {
        who: PlayerRef::You,
        zone: Zone::Graveyard,
        filter: R::Creature,
    }));
    CardDefinition {
        name: "Kraven's Last Hunt",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::Const(5) },
                    Effect::ReflexiveTrigger {
                        body: Box::new(Effect::DealDamage { to: target_filtered(R::Creature), amount: greatest }),
                    },
                ]),
            ),
            (
                2,
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
            ),
            (
                3,
                Effect::Move {
                    what: target_filtered(R::InYourGraveyard.and(R::Creature)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Ghost, Spectral Saboteur — {2}{U/B} legendary 2/2 Human Rogue Villain;
/// flash; can't be blocked.
pub fn ghost_spectral_saboteur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Unblockable],
        ..legend(creature(
            "Ghost, Spectral Saboteur",
            cost(&[generic(2), hybrid(Color::Blue, Color::Black)]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Villain],
            2,
            2,
        ))
    }
}

/// Ronin, Shadow Stalker — {2}{B} legendary 3/3 Human Rogue Hero. Pay 2 life:
/// two mana of one color for Equipment (once each turn). {T}, sacrifice an
/// Equipment attached to Ronin: target creature gets -4/-4 (sorcery speed).
pub fn ronin_shadow_stalker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                life_cost: 2,
                once_per_turn: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(Box::new(ManaPayload::AnyOneColor(Value::Const(2))), SpendRestriction::EquipmentOnly),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::AttachedToSource), 1)),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-4),
                    toughness: Value::Const(-4),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..legend(creature(
            "Ronin, Shadow Stalker",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Hero],
            3,
            3,
        ))
    }
}

/// Swordsman, Sharp Scoundrel — {1}{B} legendary 2/2 Human Hero Villain.
/// Another Villain entering under you attaches up to one target Equipment you
/// control to target creature you control; an equipped creature of yours
/// attacking connives.
pub fn swordsman_sharp_scoundrel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Villain),
                }),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Attach {
                        what: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou),
                        },
                        to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) },
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsEquipped }),
                effect: Effect::Connive { what: Selector::TriggerSource, amount: Value::ONE },
            },
        ],
        ..legend(creature(
            "Swordsman, Sharp Scoundrel",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Hero, CreatureType::Villain],
            2,
            2,
        ))
    }
}

/// Boomerang, Blade Flinger — {1}{B} legendary 1/3 Human Villain. Attacks: 1
/// damage to each opponent and you gain 1 life.
pub fn boomerang_blade_flinger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..legend(creature("Boomerang, Blade Flinger", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Villain], 1, 3))
    }
}

/// Loot, the Anomaly — {2}{B} legendary -2/4 Beast Horror. Negative power
/// assigns combat damage as though positive. Threshold — sacrifice another
/// creature or planeswalker: -2/-0 until end of turn.
pub fn loot_the_anomaly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::AssignsCombatDamageAsThoughPowerPositive],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.or(R::Planeswalker), 1)),
            condition: Some(Predicate::ThresholdActive { who: PlayerRef::You }),
            effect: Effect::PumpPT { what: Selector::This, power: Value::Const(-2), toughness: Value::Const(0), duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..legend(creature("Loot, the Anomaly", cost(&[generic(2), b()]), vec![CreatureType::Beast, CreatureType::Horror], -2, 4))
    }
}

/// Silence the Echo — {1}{B} sorcery; sacrifice a creature or planeswalker or
/// pay {3}: destroy target creature or planeswalker.
pub fn silence_the_echo() -> CardDefinition {
    CardDefinition {
        name: "Silence the Echo",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificeOrPay { filter: R::Creature.or(R::Planeswalker), pay: 3 }],
        effect: Effect::Destroy { what: target_filtered(R::Creature.or(R::Planeswalker)) },
        ..Default::default()
    }
}

/// Faceless One — {5} legendary 3/3 Enchantment Creature — Background with
/// Choose a Background; as your commander it is the color chosen before the
/// game begins (CR 903.4b).
pub fn faceless_one() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Background],
            creature_types: vec![],
            ..Default::default()
        },
        keywords: vec![Keyword::ChooseABackground],
        static_abilities: vec![StaticAbility {
            description: "If Faceless One is your commander, choose a color before the game begins. Faceless One is the chosen color.",
            effect: StaticEffect::CommanderChoosesColorBeforeGame,
        }],
        ..legend(creature("Faceless One", cost(&[generic(5)]), vec![], 3, 3))
    }
}

// ── batch 3: Rasaad yn Bashir's three decks, Faceless One // Criminal Past ──

/// Dungeoneer's Pack — {3} artifact, enters tapped. {2}, {T}, sacrifice it: you
/// take the initiative, gain 3 life, draw a card and make a Treasure; sorcery
/// speed.
pub fn dungeoneers_pack() -> CardDefinition {
    CardDefinition {
        name: "Dungeoneer's Pack",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::TakeInitiative { who: PlayerRef::You },
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(treasure_token()) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Trailblazer's Torch — {4} Equipment, equip {1}. ETB: you take the
/// initiative. Equipped creature becoming blocked deals 2 damage to each
/// creature blocking it.
pub fn trailblazers_torch() -> CardDefinition {
    CardDefinition {
        name: "Trailblazer's Torch",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        triggered_abilities: vec![etb(Effect::TakeInitiative { who: PlayerRef::You })],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::BlockingOrBlockedBySource)),
                    amount: Value::Const(2),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Harnessed Snubhorn — {3}{W} 2/5 vigilance Dinosaur. Combat damage to a
/// player returns target artifact or enchantment card from your graveyard to
/// the battlefield.
pub fn harnessed_snubhorn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(R::InYourGraveyard.and(R::Artifact.or(R::Enchantment))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..creature("Harnessed Snubhorn", cost(&[generic(3), w()]), vec![CreatureType::Dinosaur], 2, 5)
    }
}

/// Hamato Guardian Stance — {W} instant: target creature +1/+3 and flying until
/// end of turn; scry 1.
pub fn hamato_guardian_stance() -> CardDefinition {
    CardDefinition {
        name: "Hamato Guardian Stance",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::ONE, toughness: Value::Const(3), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Veteran Soldier — {1}{W} legendary Background. Your commander creatures
/// attacking a player no opponent out-lives make a tapped-and-attacking 1/1
/// Soldier at each opponent.
pub fn veteran_soldier() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    let ability = TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::All(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::IsAttackingOpponentPlayer },
            Predicate::PlayerHasMostLifeAmongOpponents { who: PlayerRef::DefendingPlayer },
        ])),
        effect: Effect::ForEachOpponent {
            body: Box::new(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(soldier),
                cleanup: Default::default(),
                defender: Some(PlayerRef::Triggerer),
            }),
        },
    };
    CardDefinition {
        name: "Veteran Soldier",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"Whenever this creature attacks a player, if no opponent has more life than that player, for each opponent, create a 1/1 white Soldier creature token that's tapped and attacking that opponent.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
                ability: Box::new(ability),
            },
        }],
        ..Default::default()
    }
}

/// Veteran Armorsmith — {W}{W} 2/3 Human Soldier; other Soldiers you control
/// get +0/+1.
pub fn veteran_armorsmith() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Soldier creatures you control get +0/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Soldier)).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 0,
                toughness: 1,
            },
        }],
        ..creature("Veteran Armorsmith", cost(&[w(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 3)
    }
}

/// Lurebound Scarecrow — {3} 4/4 artifact Scarecrow. As it enters, choose a
/// color; when you control no permanents of that color, sacrifice it.
pub fn lurebound_scarecrow() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        state_trigger: Some(StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::YouControlPermanentOfSourcesChosenColor)),
            effect: Effect::SacrificeSource,
        }),
        ..creature("Lurebound Scarecrow", cost(&[generic(3)]), vec![CreatureType::Scarecrow], 4, 4)
    }
}

/// Midnight Oil — {2}{B}{B} enchantment with seven hour counters. Your draw
/// step draws one more and removes two; your maximum hand size is its hour
/// count; each card you discard costs 1 life.
pub fn midnight_oil() -> CardDefinition {
    CardDefinition {
        name: "Midnight Oil",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Hour, Value::Const(7))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::RemoveCounter { what: Selector::This, kind: CounterType::Hour, amount: Value::Const(2) },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
                effect: Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Your maximum hand size is equal to the number of hour counters on this enchantment.",
            effect: StaticEffect::ControllerMaxHandSizeFromSourceCounters(CounterType::Hour),
        }],
        ..Default::default()
    }
}
