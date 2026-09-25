//! Commander: the cards the **Hatsune Miku** Secret Lair precon (SLD,
//! Trostani, Selesnya's Voice) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_trostani.rs`.
//!
//! Residuals (each also on its card):
//! - **Ancient Cornucopia** — "do this only once each turn" limits the trigger,
//!   not the gain: a declined gain still spends the turn's one.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R,
    Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{encore, etb, on_attack, on_you_attack, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, hybrid, w, x, Color, ManaCost};

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

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

fn gw() -> crate::mana::ManaSymbol {
    hybrid(Color::Green, Color::White)
}

fn token(name: &str, ct: CreatureType, colors: Vec<Color>, pt: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: pt,
        toughness: pt,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn citizen() -> Arc<TokenDefinition> {
    Arc::new(token("Citizen", CreatureType::Citizen, vec![Color::Green, Color::White], 1, vec![]))
}

fn mine(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn other_attackers() -> R {
    R::Creature.and(R::IsAttacking).and(R::OtherThanSource)
}

fn copy_of(source: Selector) -> Effect {
    Effect::CreateTokenCopyOf {
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
        extra_keywords: vec![],
    }
}

/// Ancient Cornucopia — casting a colored spell gains 1 life per color, once
/// each turn; taps for any color. Residual: the once-a-turn limit is on the
/// trigger, so a declined gain still spends it.
pub fn ancient_cornucopia() -> CardDefinition {
    CardDefinition {
        name: "Ancient Cornucopia",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Not(Box::new(R::Colorless))))
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "Gain 1 life for each of that spell's colors?".into(),
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ColorCountOf(Box::new(Selector::TriggerSource)),
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Angel of Indemnity — flying, lifelink; enters: a permanent card with mana
/// value 4 or less returns from your graveyard. Encore {6}{W}{W}.
pub fn angel_of_indemnity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Permanent.and(R::ManaValueAtMost(4)).from_your_graveyard()),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        activated_abilities: vec![encore(cost(&[generic(6), w(), w()]))],
        ..creature(
            "Angel of Indemnity",
            cost(&[generic(5), w()]),
            vec![CreatureType::Angel, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Blossoming Bogbeast — attacking gains 2 life, then your creatures gain
/// trample and +X/+X, X the life you gained this turn.
pub fn blossoming_bogbeast() -> CardDefinition {
    let gained = Value::LifeGainedThisTurn(PlayerRef::You);
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::GrantKeyword {
                what: mine(R::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: mine(R::Creature),
                power: gained.clone(),
                toughness: gained,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature("Blossoming Bogbeast", cost(&[generic(4), g()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Break Down — destroy target artifact or enchantment; create a Junk token.
pub fn break_down() -> CardDefinition {
    spell(
        "Break Down",
        cost(&[generic(2), g()]),
        true,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::junk_token()),
            },
        ]),
    )
}

/// Brokers Hideout — enters, is sacrificed, and fetches a basic Forest, Plains
/// or Island tapped; you gain 1 life.
pub fn brokers_hideout() -> CardDefinition {
    CardDefinition {
        name: "Brokers Hideout",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::SacrificeSource,
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(
                    R::HasLandType(LandType::Forest)
                        .or(R::HasLandType(LandType::Plains))
                        .or(R::HasLandType(LandType::Island)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Conclave Evangelist — myriad; combat damage to a player makes a token copy
/// of it.
pub fn conclave_evangelist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Myriad,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: copy_of(Selector::This),
            },
        ],
        ..creature(
            "Conclave Evangelist",
            cost(&[generic(3), gw(), gw()]),
            vec![CreatureType::Elephant, CreatureType::Cleric],
            4,
            4,
        )
    }
}

/// Ghalta and Mavren — trample; whenever you attack, choose: a tapped and
/// attacking X/X trampling Dinosaur (X the greatest power among other
/// attackers), or X 1/1 lifelink Vampires (X the other attackers).
pub fn ghalta_and_mavren() -> CardDefinition {
    let biggest = Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(other_attackers())));
    let dinosaur = TokenDefinition {
        dynamic_pt: Some((biggest.clone(), biggest)),
        ..token("Dinosaur", CreatureType::Dinosaur, vec![Color::Green], 0, vec![Keyword::Trample])
    };
    let vampire = token("Vampire", CreatureType::Vampire, vec![Color::White], 1, vec![Keyword::Lifelink]);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_you_attack(Effect::ChooseMode(vec![
            Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(dinosaur),
                cleanup: Default::default(),
                defender: None,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::EachPermanent(other_attackers()))),
                definition: Arc::new(vampire),
            },
        ]))],
        ..creature(
            "Ghalta and Mavren",
            cost(&[generic(3), g(), g(), w(), w()]),
            vec![CreatureType::Dinosaur, CreatureType::Vampire],
            12,
            12,
        )
    }
}

/// Grand Crescendo — X 1/1 Citizens; your creatures gain indestructible until
/// end of turn.
pub fn grand_crescendo() -> CardDefinition {
    spell(
        "Grand Crescendo",
        cost(&[x(), w(), w()]),
        true,
        Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::XFromCost, definition: citizen() },
            Effect::GrantKeyword {
                what: mine(R::Creature),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Halo Fountain — untapping your tapped creatures as a cost: one makes a
/// Citizen, two draw a card, fifteen win the game.
pub fn halo_fountain() -> CardDefinition {
    let untap = |mana: ManaCost, n: u32, effect: Effect| ActivatedAbility {
        tap_cost: true,
        mana_cost: mana,
        untap_others_cost: Some((R::Creature, n)),
        effect,
        ..Default::default()
    };
    CardDefinition {
        name: "Halo Fountain",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            untap(
                cost(&[w()]),
                1,
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: citizen() },
            ),
            untap(cost(&[w(), w()]), 2, Effect::Draw { who: Selector::You, amount: Value::ONE }),
            untap(cost(&[w(), w(), w(), w(), w()]), 15, Effect::WinGame { who: PlayerRef::You }),
        ],
        ..Default::default()
    }
}

/// Invincible Hymn — your life total becomes the number of cards in your
/// library.
pub fn invincible_hymn() -> CardDefinition {
    spell(
        "Invincible Hymn",
        cost(&[generic(6), w(), w()]),
        false,
        Effect::SetLifeTotal { who: Selector::You, amount: Value::LibrarySizeOf(PlayerRef::You) },
    )
}

/// Lathiel, the Bounteous Dawn — lifelink; at each end step, if you gained
/// life this turn, distribute up to that many +1/+1 counters among other
/// target creatures.
pub fn lathiel_the_bounteous_dawn() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
            ),
            effect: Effect::DistributeCounters {
                total: Value::LifeGainedThisTurn(PlayerRef::You),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature.and(R::OtherThanSource),
                max_targets: 8,
            },
        }],
        ..creature(
            "Lathiel, the Bounteous Dawn",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Unicorn],
            2,
            2,
        )
    }
}

/// Lazotep Quarry — a Desert: {C}; sacrifice a creature for any color; {X}{2},
/// sacrifice a Desert: exile a creature card with mana value X from your
/// graveyard for a 4/4 black Zombie token copy — a Zombie instead of its
/// other creature types, unlike eternalize's (sorcery speed).
pub fn lazotep_quarry() -> CardDefinition {
    CardDefinition {
        name: "Lazotep Quarry",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Desert], ..Default::default() },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[x(), generic(2)]),
                sac_other_filter: Some((R::HasLandType(LandType::Desert), 1)),
                sac_other_may_be_source: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::Target(0),
                        extra_creature_types: vec![CreatureType::Zombie],
                        extra_card_types: vec![],
                        override_pt: Some((4, 4)),
                        override_colors: Some(vec![Color::Black]),
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![],
                    },
                    Effect::SetCopiableCreatureTypes {
                        what: Selector::LastCreatedToken,
                        creature_types: vec![CreatureType::Zombie],
                    },
                    Effect::Exile {
                        what: target_filtered(
                            R::Creature.and(R::ManaValueExactlyXFromCost).from_your_graveyard(),
                        ),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Pest Infestation — destroy up to X target artifacts and/or enchantments;
/// create twice X 1/1 Pests that gain you 1 life when they die.
pub fn pest_infestation() -> CardDefinition {
    spell(
        "Pest Infestation",
        cost(&[x(), x(), g()]),
        false,
        Effect::Seq(vec![
            Effect::CapTargetsAtX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Artifact.or(R::Enchantment),
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(2))),
                definition: Arc::new(crabomination_base::tokens::stx_pest_token()),
            },
        ]),
    )
}

/// Rhys the Redeemed — {2}{G/W}, {T}: a 1/1 Elf Warrior; {4}{G/W}{G/W}, {T}:
/// copy each creature token you control.
pub fn rhys_the_redeemed() -> CardDefinition {
    let elf = token("Elf Warrior", CreatureType::Elf, vec![Color::Green, Color::White], 1, vec![]);
    let elf = TokenDefinition {
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warrior], ..Default::default() },
        ..elf
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), gw()]),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(elf) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4), gw(), gw()]),
                effect: Effect::ForEach {
                    selector: mine(R::Creature.and(R::IsToken)),
                    body: Box::new(copy_of(Selector::TriggerSource)),
                },
                ..Default::default()
            },
        ],
        ..creature("Rhys the Redeemed", cost(&[gw()]), vec![CreatureType::Elf, CreatureType::Warrior], 1, 1)
    }
}

/// Sapseep Forest — a Forest that enters tapped; {G}, {T}: gain 1 life, with
/// two or more green permanents.
pub fn sapseep_forest() -> CardDefinition {
    CardDefinition {
        name: "Sapseep Forest",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Forest], ..Default::default() },
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Green),
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: mine(R::HasColor(Color::Green)),
                    n: Value::Const(2),
                }),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Song of Freyalise — Saga. I, II: until your next turn, your creatures have
/// "{T}: Add one mana of any color." III: a +1/+1 counter on each creature you
/// control, and they gain vigilance, trample and indestructible this turn.
pub fn song_of_freyalise() -> CardDefinition {
    let any_color = ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
        ..Default::default()
    };
    // CR 611.2c — the grant reaches the creatures you control as it resolves;
    // it ends at your next upkeep, the first point in your next turn a
    // delayed trigger can reach.
    let grant = Effect::Seq(vec![
        Effect::GrantActivatedAbilityToMatching {
            filter: R::Creature.and(R::ControlledByYou),
            ability: Box::new(any_color.clone()),
            duration: Duration::Permanent,
        },
        Effect::DelayUntil {
            kind: DelayedTriggerKind::YourNextUpkeep,
            body: Box::new(Effect::RevokeGrantedActivatedAbility {
                filter: R::Creature,
                ability: Box::new(any_color),
            }),
        },
    ]);
    CardDefinition {
        name: "Song of Freyalise",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, grant.clone()),
            (2, grant),
            (
                3,
                Effect::ForEach {
                    selector: mine(R::Creature),
                    body: Box::new(Effect::Seq(vec![
                        Effect::AddCounter {
                            what: Selector::TriggerSource,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        Effect::GrantKeywords {
                            what: Selector::TriggerSource,
                            keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Indestructible],
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Soul of Eternity — power and toughness equal to your life total; encore
/// {7}{W}{W}.
pub fn soul_of_eternity() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ControllerLife),
        activated_abilities: vec![encore(cost(&[generic(7), w(), w()]))],
        ..creature("Soul of Eternity", cost(&[generic(5), w(), w()]), vec![CreatureType::Avatar], 0, 0)
    }
}
