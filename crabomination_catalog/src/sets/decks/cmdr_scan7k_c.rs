//! Commander: two-card gaps of the 2026-10-15 full-list EDHREC scan, second
//! batch — each pair closes one unseated average deck. Tests in
//! `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, mint_treasures, on_dies, prowess, sneak, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
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

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: ManaCost, host: R, extra: Vec<EnchantmentSubtype>) -> CardDefinition {
    let mut subs = vec![EnchantmentSubtype::Aura];
    subs.extend(extra);
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: subs, ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(host) },
        ..Default::default()
    }
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn tap_mana(mana: ManaCost, pool: ManaPayload) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, tap_cost: true, effect: Effect::AddMana { who: PlayerRef::You, pool }, ..Default::default() }
}

fn life_gained() -> EventSpec {
    EventSpec::new(EventKind::LifeGained, EventScope::YourControl)
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

/// Fall of Gil-galad — {1}{G} Saga. I: scry 2. II: two +1/+1 counters on
/// target creature you control. III: until end of turn target creature you
/// control gains "When this creature dies, draw two cards," then fights up to
/// one other target creature. (Garnet, Princess of Alexandria.)
pub fn fall_of_gil_galad() -> CardDefinition {
    let mine = Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) };
    let other = Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::OtherThanTargetSlot(0)) };
    saga(
        "Fall of Gil-galad",
        cost(&[generic(1), g()]),
        vec![
            (1, Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
            (
                2,
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ),
            (
                3,
                Effect::OptionalTargets {
                    min: 1,
                    body: Box::new(Effect::Seq(vec![
                        Effect::GrantTriggeredAbility {
                            what: mine.clone(),
                            trigger: Box::new(on_dies(draw(2))),
                            duration: Duration::EndOfTurn,
                        },
                        Effect::Fight { attacker: mine, defender: other },
                    ])),
                },
            ),
        ],
    )
}

/// The Huntsman's Redemption — {2}{G} Saga. I: a 3/3 green Beast. II: you
/// may sacrifice a creature; if you do, tutor a creature or basic land card
/// to hand. III: up to two target creatures each get +2/+2 and trample until
/// end of turn. (Garnet, Princess of Alexandria.)
pub fn the_huntsmans_redemption() -> CardDefinition {
    let beast = TokenDefinition {
        name: "Beast".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    };
    saga(
        "The Huntsman's Redemption",
        cost(&[generic(2), g()]),
        vec![
            (1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(beast) }),
            (
                2,
                Effect::MaySacrifice {
                    description: "Sacrifice a creature to search for a creature or basic land card?".into(),
                    filter: R::Creature,
                    count: Value::ONE,
                    then: Box::new(Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Creature.or(R::IsBasicLand),
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: None,
                },
            ),
            (
                3,
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Seq(vec![
                        Effect::PumpPT {
                            what: Selector::Target(0),
                            power: Value::Const(2),
                            toughness: Value::Const(2),
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                    ])),
                },
            ),
        ],
    )
}

/// Hall of Tagsin — land. {T}: {C}. {1}, {T}: one mana of any color. {4},
/// {T}: a tapped Powerstone token. (Ashnod, Flesh Mechanist.)
pub fn hall_of_tagsin() -> CardDefinition {
    CardDefinition {
        name: "Hall of Tagsin",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_mana(ManaCost::default(), ManaPayload::Colorless(Value::ONE)),
            tap_mana(cost(&[generic(1)]), ManaPayload::AnyOneColor(Value::ONE)),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Clay Revenant — {1} 1/2 Golem artifact creature; enters tapped. {2}{B}:
/// return it from your graveyard to your hand. (Ashnod, Flesh Mechanist.)
pub fn clay_revenant() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature("Clay Revenant", cost(&[generic(1)]), vec![CreatureType::Golem], 1, 2)
    }
}

/// Cartouche of Strength — {2}{G} Aura Cartouche on a creature you control.
/// ETB: you may have it fight target creature an opponent controls. Enchanted
/// creature gets +1/+1 and has trample. (Stangg, Echo Warrior.)
pub fn cartouche_of_strength() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: enchanted(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Enchanted creature has trample.",
                effect: StaticEffect::GrantKeyword { applies_to: enchanted(), keyword: Keyword::Trample },
            },
        ],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Have the enchanted creature fight?".into(),
            body: Box::new(Effect::Fight {
                attacker: enchanted(),
                defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            }),
        })],
        ..aura(
            "Cartouche of Strength",
            cost(&[generic(2), g()]),
            R::Creature.and(R::ControlledByYou),
            vec![EnchantmentSubtype::Cartouche],
        )
    }
}

/// Most Wanted — {2}{G} Aura with flash. Enchanted creature gets +2/+1; when
/// it dies, two Treasures. (Stangg, Echo Warrior.)
pub fn most_wanted() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +2/+1.",
            effect: StaticEffect::PumpPT { applies_to: enchanted(), power: 2, toughness: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: mint_treasures(2),
        }],
        ..aura("Most Wanted", cost(&[generic(2), g()]), R::Creature, vec![])
    }
}

/// Suburban Sanctuary — land; enters tapped. {T}: {G} or {W}. {4}, {T}:
/// surveil 1. (Silk, Web Weaver.)
pub fn suburban_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Suburban Sanctuary",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            tap_mana(ManaCost::default(), ManaPayload::Colors(vec![Color::Green])),
            tap_mana(ManaCost::default(), ManaPayload::Colors(vec![Color::White])),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Supportive Parents — {2}{G} 3/3 Human Citizen. Tap two untapped
/// creatures you control: add one mana of any color. (Silk, Web Weaver.)
pub fn supportive_parents() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature, 2)),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..creature("Supportive Parents", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Citizen], 3, 3)
    }
}

/// Hansk, Slayer Zealot — {2}{R}{G} legendary 4/4 Human Archer. Upkeep:
/// target opponent creates three 2/2 black Zombie Walkers. {T}: 2 damage to
/// target creature. Whenever a Zombie an opponent controls dies, draw.
pub fn hansk_slayer_zealot() -> CardDefinition {
    let walker = TokenDefinition {
        name: "Walker".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 1,
                    filter: R::OpponentPlayer,
                    effect: Box::new(Effect::CreateToken { who: PlayerRef::Target(0), count: Value::Const(3), definition: Arc::new(walker) }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Zombie),
                }),
                effect: draw(1),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..legend("Hansk, Slayer Zealot", cost(&[generic(2), r(), g()]), vec![CreatureType::Human, CreatureType::Archer], 4, 4)
    }
}

/// Vicious Shadows — {6}{R} enchantment. Whenever a creature dies, you may
/// have it deal damage to target player equal to the cards in that player's
/// hand. (Hansk, Adamaro.)
pub fn vicious_shadows() -> CardDefinition {
    CardDefinition {
        name: "Vicious Shadows",
        cost: cost(&[generic(6), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::MayDo {
                description: "Deal damage equal to a player's hand size?".into(),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::Player),
                    amount: Value::HandSizeOf(PlayerRef::Target(0)),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Thor Odinson — {3}{R}{W} legendary 4/4 God Warrior Hero; flying,
/// vigilance, prowess twice. (Monica Rambeau.)
pub fn thor_odinson() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Prowess],
        // Two printed prowess instances: each its own trigger.
        triggered_abilities: vec![prowess(), prowess()],
        ..legend(
            "Thor Odinson",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::God, CreatureType::Warrior, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Monica Rambeau // Photon, Living Light — {2}{W} legendary 3/3 Human Hero,
/// flying, prowess; {2}{R}{W}{W}: transform (sorcery). Back: 4/4 Elemental
/// Hero, flying, hexproof, prowess; whenever you cast a noncreature spell, a
/// +1/+1 counter on each other creature you control.
pub fn monica_rambeau() -> CardDefinition {
    let photon = CardDefinition {
        name: "Photon, Living Light",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental, CreatureType::Hero], ..Default::default() },
        color_indicator: vec![Color::Red, Color::White],
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Hexproof, Keyword::Prowess],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Not(Box::new(R::Creature)),
            }),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Prowess],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), w(), w()]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(photon)),
        ..legend("Monica Rambeau", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3)
    }
}

/// Unflinching Hortimancer — {1}{W} 2/1 Human Cleric; ward {1}; whenever you
/// gain life, a +1/+1 counter on it. (Sundial, Dawn Tyrant.)
pub fn unflinching_hortimancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: life_gained(),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Unflinching Hortimancer", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 1)
    }
}

/// Way of the Mentor — {2}{W} legendary enchantment. ETB: empower Jace 5.
/// Whenever you gain life, a loyalty counter on each planeswalker you
/// control. (Sundial, Dawn Tyrant.)
pub fn way_of_the_mentor() -> CardDefinition {
    CardDefinition {
        name: "Way of the Mentor",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(5) }),
            TriggeredAbility {
                event: life_gained(),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(R::Planeswalker.and(R::ControlledByYou)),
                    kind: CounterType::Loyalty,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Donatello, Gadget Master — {2}{U} legendary 3/2 Mutant Ninja Turtle;
/// sneak {1}{U}. Combat damage to a player: a token copy of target artifact
/// you control.
pub fn donatello_gadget_master() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(sneak(cost(&[generic(1), u()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::HasCardType(CardType::Artifact).and(R::ControlledByYou)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
        }],
        ..legend(
            "Donatello, Gadget Master",
            cost(&[generic(2), u()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            3,
            2,
        )
    }
}

/// Donatello, Mutant Mechanic — {3}{U} legendary 3/5 Mutant Ninja Turtle.
/// {T}: three +1/+1 counters on target artifact you control; a noncreature
/// one becomes a 0/0 Robot creature too (sorcery). Whenever an artifact you
/// control goes to a graveyard from the battlefield with counters, put them
/// on up to one target artifact or creature you control.
pub fn donatello_mutant_mechanic() -> CardDefinition {
    let artifact = || target_filtered(R::HasCardType(CardType::Artifact).and(R::ControlledByYou));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: artifact(), kind: CounterType::PlusOnePlusOne, amount: Value::Const(3) },
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Not(Box::new(R::Creature)) },
                    then: Box::new(Effect::BecomeCreature {
                        what: Selector::Target(0),
                        power: Value::Const(0),
                        toughness: Value::Const(0),
                        creature_types: vec![CreatureType::Robot],
                        keywords: vec![],
                        duration: Duration::Permanent,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Artifact),
            }),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::MoveAllCounters {
                    from: Selector::TriggerSource,
                    to: target_filtered(
                        R::HasCardType(CardType::Artifact).or(R::Creature).and(R::ControlledByYou),
                    ),
                }),
            },
        }],
        ..legend(
            "Donatello, Mutant Mechanic",
            cost(&[generic(3), u()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            3,
            5,
        )
    }
}
