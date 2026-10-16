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
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
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

// ── batch 4: the most-shared last cards (Glacian, Sakashima's Will, …) ──────

/// Glacian, Powerstone Engineer — {5}{U} legendary 3/6 Human Artificer,
/// partner. {T}, tap X untapped artifacts you control: look at the top X, one
/// into your hand and the rest into your graveyard.
pub fn glacian_powerstone_engineer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            tap_n_filter: Some((R::Artifact, 0)),
            tap_n_x: true,
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::XFromCost,
                rest_to_graveyard: true,
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..legend(creature(
            "Glacian, Powerstone Engineer",
            cost(&[generic(5), crate::mana::u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            6,
        ))
    }
}

/// Countervailing Winds — {2}{U} instant: counter target spell unless its
/// controller pays {1} per card in your graveyard. Cycling {2}.
pub fn countervailing_winds() -> CardDefinition {
    CardDefinition {
        name: "Countervailing Winds",
        cost: cost(&[generic(2), crate::mana::u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack),
            mana_cost: ManaCost::default(),
            exile: false,
            extra_generic: Some(Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Any }),
            if_paid: None,
        },
        ..Default::default()
    }
}

/// Flash Photography — {2}{U}{U} sorcery: a token copy of target permanent;
/// flash when it targets a permanent you control. Flashback {4}{U}{U}.
pub fn flash_photography() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Flash Photography",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), u(), u()]))],
        static_abilities: vec![StaticAbility {
            description: "You may cast this spell as though it had flash if it targets a permanent you control.",
            effect: StaticEffect::SelfFlashIfTargets { filter: R::ControlledByYou },
        }],
        effect: crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, target_filtered(R::Permanent)),
        ..Default::default()
    }
}

/// Vibrance — {3}{R/G}{R/G} 4/4 Elemental Incarnation, evoke {R/G}{R/G}. ETB
/// if {R}{R} was spent: 3 damage to any target; if {G}{G} was spent: search
/// for a land to hand and gain 2 life.
pub fn vibrance() -> CardDefinition {
    let rg = || hybrid(Color::Red, Color::Green);
    let spent = |color, effect| TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
            .with_filter(Predicate::SourceCastWithColorSpent { color, at_least: 2 }),
        effect,
    };
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[rg(), rg()]),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        triggered_abilities: vec![
            spent(Color::Red, Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) }),
            spent(
                Color::Green,
                Effect::Seq(vec![
                    Effect::Search { who: PlayerRef::You, filter: R::Land, to: ZoneDest::Hand(PlayerRef::You) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            ),
        ],
        ..creature("Vibrance", cost(&[generic(3), rg(), rg()]), vec![CreatureType::Elemental, CreatureType::Incarnation], 4, 4)
    }
}

/// Sakashima's Will — {3}{U} sorcery. Choose one (both if you control a
/// commander as you cast it): target opponent picks a creature they control and
/// you gain control of it; or each other creature you control becomes a copy of
/// a creature you choose until end of turn.
pub fn sakashimas_will() -> CardDefinition {
    use crate::effect::shortcut::{choose_one_then, chosen_one};
    let yours = || R::Creature.and(R::ControlledByYou);
    CardDefinition {
        name: "Sakashima's Will",
        cost: cost(&[generic(3), crate::mana::u()]),
        card_types: vec![CardType::Sorcery],
        modes_widen: Some(crate::card::ModesWiden::If(Predicate::YouControlACommander)),
        effect: Effect::ChooseModesCast {
            min: 1,
            max: 1,
            allow_repeats: false,
            modes: vec![
                // The opponent's pick is not targeted (hexproof doesn't stop
                // it); the target is the opponent.
                Effect::OpponentChoosesPermanentThen {
                    filter: R::Creature,
                    body: Box::new(Effect::GainControl { what: Selector::Target(0), to: None, duration: Duration::Permanent }),
                    chooser: Some(PlayerRef::Target(0)),
                    theirs: true,
                },
                choose_one_then(
                    Selector::EachPermanent(yours()),
                    PlayerRef::You,
                    Effect::BecomeCopyOfFor {
                        what: Selector::SeparatedPile { chosen: false },
                        source: chosen_one(),
                        duration: Duration::EndOfTurn,
                        non_legendary: false,
                    },
                ),
            ],
        },
        ..Default::default()
    }
}

// ── batch 5: the next most-shared last cards ─────────────────────────────────

/// Serra's Guardian — {4}{W}{W} 5/5 Angel, flying, vigilance; other creatures
/// you control have vigilance.
pub fn serras_guardian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Vigilance,
            },
        }],
        ..creature("Serra's Guardian", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 5, 5)
    }
}

/// Alpine Meadow — Snow Land — Mountain Plains, enters tapped.
pub fn alpine_meadow() -> CardDefinition {
    let mut d = crate::sets::dual_land_with(
        "Alpine Meadow",
        crate::card::LandType::Mountain,
        crate::card::LandType::Plains,
        Color::Red,
        Color::White,
        vec![],
    );
    d.supertypes.push(Supertype::Snow);
    d.static_abilities.push(crate::sets::enters_tapped());
    d
}

/// Dark Fortress — Land. {T}: {C}. {T}: {B} or {R}, only if it entered this
/// turn or you control a basic land.
pub fn dark_fortress() -> CardDefinition {
    let gated = |c: Color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![c]) },
        condition: Some(Predicate::Any(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn },
            Predicate::SelectorExists(Selector::EachPermanent(R::IsBasicLand.and(R::ControlledByYou))),
        ])),
        ..Default::default()
    };
    CardDefinition {
        name: "Dark Fortress",
        card_types: vec![CardType::Land],
        activated_abilities: vec![crate::sets::tap_add_colorless(), gated(Color::Black), gated(Color::Red)],
        ..Default::default()
    }
}

/// Timetwister — {2}{U} sorcery: each player shuffles hand and graveyard into
/// their library, then draws seven.
pub fn timetwister() -> CardDefinition {
    CardDefinition {
        name: "Timetwister",
        cost: cost(&[generic(2), crate::mana::u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
            Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(7) },
        ]),
        ..Default::default()
    }
}

/// Stoic Rebuttal — {1}{U}{U} instant: counter target spell; metalcraft, {1}
/// less.
pub fn stoic_rebuttal() -> CardDefinition {
    CardDefinition {
        name: "Stoic Rebuttal",
        cost: cost(&[generic(1), crate::mana::u(), crate::mana::u()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "Metalcraft — This spell costs {1} less to cast if you control three or more artifacts.",
            effect: StaticEffect::SelfCostReducedIfPredicate {
                amount: 1,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                    n: Value::Const(3),
                },
            },
        }],
        effect: Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
        ..Default::default()
    }
}

/// Avenging Hunter — {4}{G} 5/4 Dragon Ranger, trample; ETB: take the initiative.
pub fn avenging_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::TakeInitiative { who: PlayerRef::You })],
        ..creature("Avenging Hunter", cost(&[generic(4), g()]), vec![CreatureType::Dragon, CreatureType::Ranger], 5, 4)
    }
}

/// Elvish Doomsayer — {1}{B} 1/1 Elf Shaman; dies: each opponent discards a card.
pub fn elvish_doomsayer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            random: false,
        })],
        ..creature("Elvish Doomsayer", cost(&[generic(1), b()]), vec![CreatureType::Elf, CreatureType::Shaman], 1, 1)
    }
}

/// Shadows of the Past — {1}{B} enchantment. A creature dying scries 1; {4}{B}:
/// drain each opponent for 2, only with four or more creature cards in your
/// graveyard.
pub fn shadows_of_the_past() -> CardDefinition {
    CardDefinition {
        name: "Shadows of the Past",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            condition: Some(Predicate::ValueAtLeast(
                Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
                Value::Const(4),
            )),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Darklight Phoenix — {3}{B} 3/2 flying, haste Phoenix. At the beginning of
/// combat on your turn, if two or more creatures died this turn, it returns
/// from your graveyard to the battlefield.
pub fn darklight_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::FromYourGraveyard).with_filter(
                Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(2) },
                ]),
            ),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false } },
        }],
        ..creature("Darklight Phoenix", cost(&[generic(3), b()]), vec![CreatureType::Phoenix], 3, 2)
    }
}

/// Reason // Believe — {U} sorcery: scry 3. Aftermath {4}{G} sorcery: look at
/// the top card; a creature may go onto the battlefield, else into your hand.
pub fn reason_believe() -> CardDefinition {
    CardDefinition {
        name: "Reason // Believe",
        cost: cost(&[crate::mana::u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(4), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    picked_matching_to_battlefield: Some(R::Creature),
                    ..Default::default()
                })),
            },
            fuse: false,
            aftermath: true,
        })),
        ..Default::default()
    }
}

/// Elvish Dreadlord — {3}{B}{B} 3/3 deathtouch Zombie Elf. Dies: non-Elf
/// creatures get -3/-3 until end of turn. Encore {5}{B}{B}.
pub fn elvish_dreadlord() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Elf).negate())),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![crate::effect::shortcut::encore(cost(&[generic(5), b(), b()]))],
        ..creature("Elvish Dreadlord", cost(&[generic(3), b(), b()]), vec![CreatureType::Zombie, CreatureType::Elf], 3, 3)
    }
}

/// Tomb of Horrors Adventurer — {5}{U} 4/4 Elf Monk. ETB: take the initiative.
/// Your second spell each turn is copied (twice once you've completed a
/// dungeon); new targets may be chosen.
pub fn tomb_of_horrors_adventurer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::TakeInitiative { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(2) }),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(Value::DungeonsCompleted, Value::ONE),
                    then: Box::new(Effect::CopySpell { what: Selector::TriggerSource, count: Value::Const(2) }),
                    else_: Box::new(Effect::CopySpell { what: Selector::TriggerSource, count: Value::ONE }),
                },
            },
        ],
        ..creature("Tomb of Horrors Adventurer", cost(&[generic(5), crate::mana::u()]), vec![CreatureType::Elf, CreatureType::Monk], 4, 4)
    }
}

// ── batch 6 ─────────────────────────────────────────────────────────────────

/// Sulfurous Mire — Snow Land — Swamp Mountain, enters tapped.
pub fn sulfurous_mire() -> CardDefinition {
    let mut d = crate::sets::dual_land_with(
        "Sulfurous Mire",
        crate::card::LandType::Swamp,
        crate::card::LandType::Mountain,
        Color::Black,
        Color::Red,
        vec![],
    );
    d.supertypes.push(Supertype::Snow);
    d.static_abilities.push(crate::sets::enters_tapped());
    d
}

/// Giant's Boulder — {1} artifact. ETB scry 2; {1}, {T}: any color; {7}, {T},
/// sacrifice it: destroy target permanent.
pub fn giants_boulder() -> CardDefinition {
    CardDefinition {
        name: "Giant's Boulder",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(7)]),
                effect: Effect::Destroy { what: target_filtered(R::Permanent) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Warg Tactics — {1}{G} instant. Choose one: destroy target creature with
/// flying; or a +1/+1 counter on target creature you control, which gains
/// trample and hexproof until end of turn.
pub fn warg_tactics() -> CardDefinition {
    CardDefinition {
        name: "Warg Tactics",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))) },
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
            ]),
        ]),
        ..Default::default()
    }
}

/// Konstrari Charm — {R}{G} instant. Choose one: 6 damage to target creature
/// with flying; two +1/+1 counters on target creature and trample this turn;
/// or add {C}{C}{C}.
pub fn konstrari_charm() -> CardDefinition {
    CardDefinition {
        name: "Konstrari Charm",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage { to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))), amount: Value::Const(6) },
            Effect::Seq(vec![
                Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
            Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(3)) },
        ]),
        ..Default::default()
    }
}

/// Imoen, Mystic Trickster — {2}{U} legendary 2/3 Human Rogue Wizard, ward {2},
/// Choose a Background. Your end step with the initiative draws a card, two
/// once you've completed a dungeon.
pub fn imoen_mystic_trickster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(2)]))), Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::HasInitiative { who: PlayerRef::You }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Sum(vec![
                    Value::ONE,
                    Value::OneIf(Box::new(Predicate::ValueAtLeast(Value::DungeonsCompleted, Value::ONE))),
                ]),
            },
        }],
        ..legend(creature(
            "Imoen, Mystic Trickster",
            cost(&[generic(2), crate::mana::u()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Wizard],
            2,
            3,
        ))
    }
}

/// Vrock — {3}{B}{B} 3/3 flying Bird Demon. Toxic Spores: your end step, if a
/// permanent you controlled left the battlefield this turn, each opponent
/// loses 3 life.
pub fn vrock() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
        }],
        ..creature("Vrock", cost(&[generic(3), b(), b()]), vec![CreatureType::Bird, CreatureType::Demon], 3, 3)
    }
}

/// Glamer Gifter — {1}{U} 1/2 flash, flying Faerie Wizard. ETB: up to one
/// other target creature is a base 4/4 with all creature types this turn.
pub fn glamer_gifter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::Creature.and(R::OtherThanSource),
            effect: Box::new(Effect::Seq(vec![
                Effect::SetBasePtLive { what: Selector::Target(0), power: Value::Const(4), toughness: Value::Const(4), duration: Duration::EndOfTurn },
                Effect::GainAllCreatureTypes { what: Selector::Target(0), duration: Duration::EndOfTurn },
            ])),
        })],
        ..creature("Glamer Gifter", cost(&[generic(1), crate::mana::u()]), vec![CreatureType::Faerie, CreatureType::Wizard], 1, 2)
    }
}

/// Prismatic Undercurrents — {3}{G} enchantment. Vivid: ETB, search for up to
/// X basic lands to hand (X = colors among your permanents). One extra land
/// each turn.
pub fn prismatic_undercurrents() -> CardDefinition {
    CardDefinition {
        name: "Prismatic Undercurrents",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
            count: Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::ControlledByYou))),
        })],
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        ..Default::default()
    }
}

/// Ajani Unrelenting — {4}{R}{R} Ajani, loyalty 5. Each loyalty activation
/// makes a 2/2 Cadet. +1: team +1/+0 and haste; −2: discard your hand, draw
/// one per creature you control; −3: 4 damage to each creature but your tokens.
pub fn ajani_unrelenting() -> CardDefinition {
    let cadet = TokenDefinition {
        name: "Cadet".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Ajani Unrelenting",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Ajani], ..Default::default() },
        base_loyalty: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivatedWith(crate::effect::ActivationTrait::Loyalty), EventScope::YourControl),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(cadet) },
        }],
        loyalty_abilities: vec![
            crate::card::LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::PumpPT { what: team(), power: Value::ONE, toughness: Value::ZERO, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: team(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::Const(100), random: false },
                    Effect::Draw { who: Selector::You, amount: Value::count(team()) },
                ]),
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::IsToken.and(R::ControlledByYou).negate())),
                    amount: Value::Const(4),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Earth's Mightiest Heroes — {4}{G}{G} sorcery, teamwork 5. Reveal the top
/// eight: one creature onto the battlefield (any number with teamwork); the
/// rest into your graveyard.
pub fn earths_mightiest_heroes() -> CardDefinition {
    let pick = |take: Value| {
        Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(8),
            pick_filter: Some(R::Creature),
            take: Some(take),
            to_battlefield: true,
            optional: true,
            rest_to_graveyard: true,
            ..Default::default()
        }))
    };
    CardDefinition {
        name: "Earth's Mightiest Heroes",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower { filter: R::Creature.and(R::ControlledByYou), power: 5 }),
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(pick(Value::Const(8))),
            else_: Box::new(pick(Value::ONE)),
        },
        ..Default::default()
    }
}

// ── batch 7 ─────────────────────────────────────────────────────────────────

/// Abby, Merciless Soldier — {1}{R}{G} legendary 4/4 Human Survivor,
/// Partner—Survivors. Cast: a 1/1 black Fungus Zombie named Cordyceps Infected
/// per mana spent; she enters under an opponent of your choice.
pub fn abby_merciless_soldier() -> CardDefinition {
    let infected = TokenDefinition {
        name: "Cordyceps Infected".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fungus, CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::PartnerLabel("Survivors".into())],
        enters_under_opponent_control: true,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::CastSpellManaSpent, definition: Arc::new(infected) },
        }],
        ..legend(creature(
            "Abby, Merciless Soldier",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Human, CreatureType::Survivor],
            4,
            4,
        ))
    }
}

/// Gale's Redirection — {3}{U}{U} instant. Exile target spell, roll a d20 plus
/// its mana value: 1-14, you may cast it while it stays exiled, spending mana
/// as any color; 15+, you may cast it free while it stays exiled.
pub fn gales_redirection() -> CardDefinition {
    use crate::mana::u;
    let grant = |free: bool| Effect::GrantMayPlay {
        what: Selector::CardExiledWithSource,
        duration: crate::card::MayPlayDuration::WhileExiled,
        to_owner: false,
        exile_after: false,
        pay_own_cost: !free,
        any_color: !free,
    };
    CardDefinition {
        name: "Gale's Redirection",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::MoveSpellToZone {
                what: target_filtered(R::IsSpellOnStack),
                zone: crate::effect::CounteredSpellZone::ExileWithSource,
            },
            Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::ManaValueOf(Box::new(Selector::CardExiledWithSource)),
                reroll_at_most: 0,
                ignore_lowest: 0,
                results: vec![(1, 14, grant(false)), (15, u8::MAX, grant(true))],
                on_doubles: None,
            },
        ]),
        ..Default::default()
    }
}

/// Bilbo, Thief in the Night — {1}{U} legendary 2/2 Halfling Rogue. Spells you
/// cast from anywhere but your hand cost {1} less. Attacking, you may cast an
/// artifact, instant or sorcery from your graveyard (paying for it); an instant
/// or sorcery cast this way is exiled instead of going to the graveyard.
pub fn bilbo_thief_in_the_night() -> CardDefinition {
    use crate::effect::shortcut::{choose_one_then, chosen_one, on_attack};
    let castable = R::Artifact.or(R::HasCardType(CardType::Instant)).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells you cast from anywhere other than your hand cost {1} less to cast.",
            effect: StaticEffect::NonHandCastCostReduction { amount: 1 },
        }],
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Cast an artifact, instant or sorcery spell from your graveyard?".into(),
            body: Box::new(choose_one_then(
                Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: castable },
                PlayerRef::You,
                Effect::CastWithoutPayingImmediate {
                    what: chosen_one(),
                    source_zone: Zone::Graveyard,
                    exile_after: true,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: true,
                },
            )),
        })],
        ..legend(creature(
            "Bilbo, Thief in the Night",
            cost(&[generic(1), crate::mana::u()]),
            vec![CreatureType::Halfling, CreatureType::Rogue],
            2,
            2,
        ))
    }
}

/// Lukka, Coppercoat Outcast — {3}{R}{R} Lukka, loyalty 5. +1: exile the top
/// three; creature cards among them may be cast from exile while you control a
/// Lukka. −2: exile target creature you control, then reveal until a creature
/// card of greater mana value, put it onto the battlefield, rest on the bottom.
/// −7: each creature you control deals damage equal to its power to each
/// opponent.
pub fn lukka_coppercoat_outcast() -> CardDefinition {
    use crate::card::{LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype};
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Lukka, Coppercoat Outcast",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Lukka], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::ExileTopOfLibrary { who: Selector::You, amount: Value::Const(3), link_to_source: false, face_down: false },
                    Effect::GrantMayPlay {
                        what: Selector::ExiledThisResolution { filter: R::Creature },
                        duration: MayPlayDuration::WhileExiledIfHolderControlsPlaneswalker { subtype: PlaneswalkerSubtype::Lukka },
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::Move { what: target_filtered(R::Creature.and(R::ControlledByYou)), to: ZoneDest::Exile },
                    Effect::RevealUntilCreatureAboveLastMovedToBattlefield,
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::EachDealsDamageEqualToPower { dealers: yours(), target: Selector::Player(PlayerRef::EachOpponent) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── batch 8 ─────────────────────────────────────────────────────────────────

fn golem_4_4() -> TokenDefinition {
    TokenDefinition {
        name: "Golem".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        ..Default::default()
    }
}

/// Golden Guardian // Gold-Forge Garrison — {4} 4/4 defender Golem. {2}: it
/// fights another target creature you control; when it dies this turn, it
/// returns transformed under your control. Back: a land tapping for two mana
/// of one color; {4}, {T}: a 4/4 Golem.
pub fn golden_guardian() -> CardDefinition {
    let garrison = CardDefinition {
        name: "Gold-Forge Garrison",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(2)) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(golem_4_4()) },
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            // The watch goes first: the fight's damage can kill it before
            // the next step of this resolution would register it.
            effect: Effect::Seq(vec![
                Effect::WhenSourceDiesThisTurn { body: Box::new(Effect::ExileSelfReturnTransformed) },
                Effect::Fight {
                    attacker: Selector::This,
                    defender: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(garrison)),
        ..creature("Golden Guardian", cost(&[generic(4)]), vec![CreatureType::Golem], 4, 4)
    }
}

/// Rootwire Amalgam — {5} 5/5 Golem; prototype {1}{G} 2/3. {3}{G}{G},
/// sacrifice it: an X/X Golem with haste this turn, X three times its power
/// (sorcery speed).
pub fn rootwire_amalgam() -> CardDefinition {
    let x = || Value::Times(Box::new(Value::Const(3)), Box::new(Value::SacrificedPower));
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        prototype: Some(Box::new(crate::card::Prototype { cost: cost(&[generic(1), g()]), power: 2, toughness: 3 })),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), g()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition { power: 0, toughness: 0, dynamic_pt: Some((x(), x())), ..golem_4_4() }),
                },
                Effect::GrantKeyword { what: Selector::LastCreatedTokens, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Rootwire Amalgam", cost(&[generic(5)]), vec![CreatureType::Golem], 5, 5)
    }
}

/// Talas Explorer — {1}{U} 1/1 flying Human Pirate Scout; ETB: look at target
/// opponent's hand.
pub fn talas_explorer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::LookAtHand { who: target_filtered(R::OpponentPlayer) })],
        ..creature(
            "Talas Explorer",
            cost(&[generic(1), crate::mana::u()]),
            vec![CreatureType::Human, CreatureType::Pirate, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Talas Scout — {1}{U} 1/2 flying Human Pirate Scout.
pub fn talas_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature(
            "Talas Scout",
            cost(&[generic(1), crate::mana::u()]),
            vec![CreatureType::Human, CreatureType::Pirate, CreatureType::Scout],
            1,
            2,
        )
    }
}

/// Unpredictable Cyclone — {3}{R}{R} enchantment, cycling {2}. Another nonland
/// card's cycling draw becomes: exile from the top until a card sharing a card
/// type with it, which you may cast free; the rest go to the bottom.
pub fn unpredictable_cyclone() -> CardDefinition {
    CardDefinition {
        name: "Unpredictable Cyclone",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        static_abilities: vec![StaticAbility {
            description: "If a cycling ability of another nonland card would cause you to draw a card, instead exile cards from the top of your library until you exile a card that shares a card type with the cycled card. You may cast that card without paying its mana cost. Then put the exiled cards that weren't cast this way on the bottom of your library in a random order.",
            effect: StaticEffect::CyclingDrawBecomesSharedTypeCast,
        }],
        ..Default::default()
    }
}

/// Shimmerwilds Growth — {1}{G} Aura, enchant land. As it enters, choose a
/// color: the land is that color, and tapping it for mana adds one more of it.
pub fn shimmerwilds_growth() -> CardDefinition {
    CardDefinition {
        name: "Shimmerwilds Growth",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter: R::Land } },
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted land is the chosen color.",
                effect: StaticEffect::SetColorOfMatchingToChosen { applies_to: Selector::AttachedTo(Box::new(Selector::This)) },
            },
            StaticAbility {
                description: "Whenever enchanted land is tapped for mana, its controller adds an additional one mana of the chosen color.",
                effect: StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: true,
                    filter: R::Any,
                    extra: crate::effect::ExtraManaKind::ChosenColor,
                    while_monarch: false,
                },
            },
        ],
        ..Default::default()
    }
}

// ── batch 9 ─────────────────────────────────────────────────────────────────

/// Demon of Dark Schemes — {3}{B}{B}{B} 5/5 flying Demon. ETB: all other
/// creatures -2/-2 this turn. Another creature dying gives you {E}. {2}{B},
/// pay {E}{E}{E}{E}: put target creature card from a graveyard onto the
/// battlefield under your control tapped.
pub fn demon_of_dark_schemes() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource }),
                effect: Effect::AddEnergy(Value::ONE),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            energy_cost: 4,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature("Demon of Dark Schemes", cost(&[generic(3), b(), b(), b()]), vec![CreatureType::Demon], 5, 5)
    }
}

/// Loot, the Nexus — {2}{G} legendary 2/1 Beast Noble. {T}: one mana of a
/// color per distinct power among creatures you control.
pub fn loot_the_nexus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::DistinctPowersAmongCreaturesControlled(PlayerRef::You)),
            },
            ..Default::default()
        }],
        ..legend(creature("Loot, the Nexus", cost(&[generic(2), g()]), vec![CreatureType::Beast, CreatureType::Noble], 2, 1))
    }
}

/// Plunder the Trollshaws — {1}{U} instant: draw a card, two if cast from a
/// graveyard. Flashback {3}{U}.
pub fn plunder_the_trollshaws() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Plunder the Trollshaws",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u()]))],
        effect: Effect::If {
            cond: Predicate::CastFromGraveyard,
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
            else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        },
        ..Default::default()
    }
}

/// Yuriko, Hope from the Shadows — {U} legendary 1/1 flash Human Ninja. ETB,
/// choose one: target creature gets -X/-0 this turn (X = cards in your
/// graveyard); or surveil 2.
pub fn yuriko_hope_from_the_shadows() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Times(
                    Box::new(Value::Const(-1)),
                    Box::new(Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Any }),
                ),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
        ]))],
        ..legend(creature(
            "Yuriko, Hope from the Shadows",
            cost(&[crate::mana::u()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            1,
            1,
        ))
    }
}

/// Garruk, Curse Breaker — {3}{G}{G} Garruk, loyalty 5. A power-4+ creature
/// entering under you draws a card. +2: untap up to two target lands; −3: a
/// 4/4 trample Beast; −4: until your next turn, creatures attacking your
/// opponents get +2/+2 and trample.
pub fn garruk_curse_breaker() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    let beast = TokenDefinition {
        name: "Beast".into(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Garruk, Curse Breaker",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Garruk], ..Default::default() },
        base_loyalty: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::PowerAtLeast(4)),
            }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 2,
                    filter: R::Land,
                    effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(beast) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::TriggersUntilYourNextTurn {
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::IsAttackingOpponentPlayer,
                        }),
                        effect: Effect::Seq(vec![
                            Effect::PumpPT {
                                what: Selector::TriggerSource,
                                power: Value::Const(2),
                                toughness: Value::Const(2),
                                duration: Duration::EndOfTurn,
                            },
                            Effect::GrantKeyword { what: Selector::TriggerSource, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                        ]),
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Founding the Third Path — {1}{U} read-ahead Saga. I: you may cast an
/// instant or sorcery with mana value 1 or 2 from your hand free. II: target
/// player mills four. III: exile target instant or sorcery card from your
/// graveyard; you may cast a copy of it (paying for it).
pub fn founding_the_third_path() -> CardDefinition {
    use crate::effect::shortcut::{choose_one_then, chosen_one};
    let spell = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Founding the Third Path",
        cost: cost(&[generic(1), crate::mana::u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        read_ahead: true,
        saga_chapters: vec![
            (
                1,
                choose_one_then(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: spell().and(R::ManaValueAtLeast(1)).and(R::ManaValueAtMost(2)),
                    },
                    PlayerRef::You,
                    Effect::CastWithoutPayingImmediate {
                        what: chosen_one(),
                        source_zone: Zone::Hand,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    },
                ),
            ),
            (2, Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(4) }),
            (
                3,
                Effect::Seq(vec![
                    Effect::Move { what: target_filtered(R::InYourGraveyard.and(spell())), to: ZoneDest::Exile },
                    Effect::CastWithoutPayingImmediate {
                        what: Selector::LastMoved,
                        source_zone: Zone::Exile,
                        exile_after: false,
                        copy: true,
                        reduce_generic: 0,
                        pay_own_cost: true,
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Raph & Leo, Sibling Rivals — {1}{R/W}{R/W} legendary 2/4 Mutant Ninja
/// Turtle. Attacking in the turn's first combat: untap one or two target
/// attacking creatures, and there is an additional combat phase.
pub fn raph_and_leo_sibling_rivals() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::IsFirstCombatPhaseThisTurn),
            effect: Effect::Seq(vec![
                Effect::ApplyToTargets {
                    min_targets: 1,
                    max_targets: 2,
                    filter: R::Creature.and(R::IsAttacking),
                    effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
                },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        }],
        ..legend(creature(
            "Raph & Leo, Sibling Rivals",
            cost(&[generic(1), rw(), rw()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            2,
            4,
        ))
    }
}

// ── batch 10: the short-text two- and three-deck last cards ─────────────────

/// Splicer's Skill — {2}{W} sorcery: a 3/3 Phyrexian Golem artifact creature.
/// Splice onto instant or sorcery {3}{W}.
pub fn splicers_skill() -> CardDefinition {
    CardDefinition {
        name: "Splicer's Skill",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::SpliceOntoInstantOrSorcery(cost(&[generic(3), w()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(TokenDefinition {
                name: "Phyrexian Golem".into(),
                power: 3,
                toughness: 3,
                subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem], ..Default::default() },
                ..golem_4_4()
            }),
        },
        ..Default::default()
    }
}

/// Lifecrafter's Gift — {3}{G} instant: a +1/+1 counter on target creature,
/// then one on each creature you control that has a +1/+1 counter.
pub fn lifecrafters_gift() -> CardDefinition {
    CardDefinition {
        name: "Lifecrafter's Gift",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Stone by Sunlight — {1}{W} instant. Choose one: destroy target creature
/// with power 4 or greater; or target creature becomes an artifact in addition
/// and gains indestructible until end of turn.
pub fn stone_by_sunlight() -> CardDefinition {
    CardDefinition {
        name: "Stone by Sunlight",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerAtLeast(4))) },
            Effect::Seq(vec![
                Effect::AddCardTypeIndefinitely {
                    what: target_filtered(R::Creature),
                    card_type: CardType::Artifact,
                    until_eot: true,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            ]),
        ]),
        ..Default::default()
    }
}

/// Pore Over the Pages — {3}{U}{U} sorcery: draw three, untap up to two lands,
/// then discard a card.
pub fn pore_over_the_pages() -> CardDefinition {
    CardDefinition {
        name: "Pore Over the Pages",
        cost: cost(&[generic(3), crate::mana::u(), crate::mana::u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Untap { what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)), up_to: Some(Value::Const(2)) },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ]),
        ..Default::default()
    }
}

/// Goblin Bangchuckers — {2}{R}{R} 2/2 Goblin Warrior. {T}: flip a coin; win,
/// 2 damage to any target; lose, 2 damage to itself.
pub fn goblin_bangchuckers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(2) }),
                on_tails: Box::new(Effect::DealDamage { to: Selector::This, amount: Value::Const(2) }),
            },
            ..Default::default()
        }],
        ..creature("Goblin Bangchuckers", cost(&[generic(2), r(), r()]), vec![CreatureType::Goblin, CreatureType::Warrior], 2, 2)
    }
}

/// Darksteel Sentinel — {6} 3/3 artifact Golem: flash, vigilance, indestructible.
pub fn darksteel_sentinel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flash, Keyword::Vigilance, Keyword::Indestructible],
        ..creature("Darksteel Sentinel", cost(&[generic(6)]), vec![CreatureType::Golem], 3, 3)
    }
}

/// Implement of Ferocity — {1} artifact. {G}, sacrifice it: a +1/+1 counter on
/// target creature (sorcery speed). Put into a graveyard from the battlefield:
/// draw a card.
pub fn implement_of_ferocity() -> CardDefinition {
    CardDefinition {
        name: "Implement of Ferocity",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Koma's Faithful — {2}{B} 3/1 lifelink Elf Cleric; dies: each player mills three.
pub fn komas_faithful() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Mill {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(3),
        })],
        ..creature("Koma's Faithful", cost(&[generic(2), b()]), vec![CreatureType::Elf, CreatureType::Cleric], 3, 1)
    }
}

/// Primal Druid — {1}{G} 0/3 Human Druid; dies: you may search for a basic
/// land and put it onto the battlefield tapped.
pub fn primal_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::MayDo {
            description: "Search for a basic land and put it onto the battlefield tapped?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
        })],
        ..creature("Primal Druid", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Druid], 0, 3)
    }
}

/// Riddlemaster Sphinx — {4}{U}{U} 5/5 flying Sphinx; ETB: you may return
/// target creature an opponent controls to its owner's hand.
pub fn riddlemaster_sphinx() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..creature("Riddlemaster Sphinx", cost(&[generic(4), crate::mana::u(), crate::mana::u()]), vec![CreatureType::Sphinx], 5, 5)
    }
}

/// Alchemist's Vial — {2} artifact. ETB: draw a card. {1}, {T}, sacrifice it:
/// target creature can't attack or block this turn.
pub fn alchemists_vial() -> CardDefinition {
    CardDefinition {
        name: "Alchemist's Vial",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::CantAttack, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gluttonous Slime — {2}{G} 2/2 Ooze, flash, devour 1.
pub fn gluttonous_slime() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Devour(1)],
        ..creature("Gluttonous Slime", cost(&[generic(2), g()]), vec![CreatureType::Ooze], 2, 2)
    }
}

/// Soul Summons — {1}{W} sorcery: manifest the top card of your library.
pub fn soul_summons() -> CardDefinition {
    CardDefinition {
        name: "Soul Summons",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Manifest { who: PlayerRef::You, amount: Value::ONE },
        ..Default::default()
    }
}

/// Nature's Way — {1}{G} sorcery: target creature you control gains vigilance
/// and trample this turn and deals damage equal to its power to target
/// creature you don't control.
pub fn natures_way() -> CardDefinition {
    CardDefinition {
        name: "Nature's Way",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::DealDamageEqualToPower {
                source: Selector::Target(0),
                target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou.negate()) },
            },
        ]),
        ..Default::default()
    }
}

/// Pummeler for Hire — {4}{G} 4/4 Giant Mercenary: vigilance, reach, ward {2}.
/// ETB: gain life equal to the greatest power among Giants you control.
pub fn pummeler_for_hire() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Reach,
            Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(2)]))),
        ],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(R::HasCreatureType(CreatureType::Giant)))),
        })],
        ..creature("Pummeler for Hire", cost(&[generic(4), g()]), vec![CreatureType::Giant, CreatureType::Mercenary], 4, 4)
    }
}

/// Center Soul — {1}{W} instant, rebound: target creature you control gains
/// protection from the color of your choice until end of turn.
pub fn center_soul() -> CardDefinition {
    CardDefinition {
        name: "Center Soul",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Rebound],
        effect: Effect::GrantProtectionFromChosenColor {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Price of Loyalty — {2}{R} sorcery: gain control of target creature until end
/// of turn, untap it, haste; +2/+0 too if Treasure mana was spent on it.
pub fn price_of_loyalty() -> CardDefinition {
    CardDefinition {
        name: "Price of Loyalty",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::TreasureManaSpentToCastSource, Value::ONE),
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Noggle the Mind — {1}{U} flash Aura: enchanted creature loses all abilities
/// and is a colorless 1/1 Noggle (no other colors or creature types).
pub fn noggle_the_mind() -> CardDefinition {
    CardDefinition {
        name: "Noggle the Mind",
        cost: cost(&[generic(1), crate::mana::u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((1, 1)),
            set_creature_types: Some(vec![CreatureType::Noggle]),
            set_colors: Some(vec![]),
            remove_abilities: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Dreamcaller Siren — {2}{U}{U} 3/3 flash, flying Siren Pirate that can block
/// only fliers. ETB with another Pirate: tap up to two target nonland permanents.
pub fn dreamcaller_siren() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::CanBlockOnlyFlying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::SelectorExists(
                Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Pirate).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
            )),
            effect: Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 2,
                filter: R::Permanent.and(R::Land.negate()),
                effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
            },
        }],
        ..creature(
            "Dreamcaller Siren",
            cost(&[generic(2), crate::mana::u(), crate::mana::u()]),
            vec![CreatureType::Siren, CreatureType::Pirate],
            3,
            3,
        )
    }
}
