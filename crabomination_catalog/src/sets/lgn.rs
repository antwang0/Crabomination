//! Legions (LGN) — the all-creature set: Amplify, Morph turn-up triggers,
//! Provoke, cycling creatures and the Sliver lords. Tests in `classic_sets/lgn`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Zone,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Selector,
    StaticEffect, Value, ZoneDest,
    shortcut::{draw, etb, target_any, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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

/// CR 702.38 — Amplify N: enters with N counters per revealed card of `kind`
/// (the reveal is automatic; the engine counts the matching hand).
fn amplify(n: i32, kinds: &[CreatureType]) -> Option<(CounterType, Value)> {
    let filter = kinds
        .iter()
        .map(|&k| R::HasCreatureType(k))
        .reduce(|a, b| a.or(b))
        .unwrap_or(R::Creature);
    Some((
        CounterType::PlusOnePlusOne,
        Value::Times(
            Box::new(Value::CardsInHandMatching { who: PlayerRef::You, filter }),
            Box::new(Value::Const(n)),
        ),
    ))
}

/// "When this creature is turned face up, [effect]." (CR 708.8)
fn on_turn_up(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
        effect,
    }
}

/// "When you cycle this card, [effect]." — the Gempalm cycle's may-trigger.
fn on_cycle(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
        effect: Effect::MayDo { description: "Use the cycling trigger?".into(), body: Box::new(effect) },
    }
}

/// CR 702.39 — Provoke: "whenever this creature attacks, you may have target
/// creature defending player controls untap and block it if able."
fn provoke() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::MayDo {
            description: "Provoke a blocker?".into(),
            body: Box::new(Effect::Provoke {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            }),
        },
    }
}

/// The number of battlefield permanents matching `filter`.
fn count_on_battlefield(filter: R) -> Value {
    Value::CountOf(Box::new(Selector::EachPermanent(filter)))
}

/// "You control at least one [filter]."
fn you_control(filter: R) -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(filter.and(R::ControlledByYou)),
        n: Value::ONE,
    }
}

fn all_of(kind: CreatureType) -> Selector {
    Selector::EachPermanent(R::HasCreatureType(kind))
}

fn sliver(name: &'static str, c: ManaCost, p: i32, t: i32) -> CardDefinition {
    creature(name, c, vec![CreatureType::Sliver], p, t)
}

/// "{1}: This creature becomes the creature type of your choice until end of
/// turn." — the Mistform activation.
fn become_chosen_type() -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        effect: Effect::BecomeChosenCreatureType {
            what: Selector::This,
            duration: Duration::EndOfTurn,
            excluded: vec![],
        },
        ..Default::default()
    }
}

fn your_creatures_of(kind: CreatureType) -> Selector {
    Selector::EachPermanent(R::HasCreatureType(kind).and(R::ControlledByYou))
}

fn bear_token() -> TokenDefinition {
    TokenDefinition {
        name: "Bear".to_string(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        ..Default::default()
    }
}

fn sliver_token() -> TokenDefinition {
    TokenDefinition {
        name: "Sliver".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sliver], ..Default::default() },
        ..Default::default()
    }
}

fn goblin_token() -> TokenDefinition {
    TokenDefinition {
        name: "Goblin".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Akroma, Angel of Wrath — the keyword pile.
pub fn akroma_angel_of_wrath() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![
            Keyword::Flying,
            Keyword::FirstStrike,
            Keyword::Vigilance,
            Keyword::Trample,
            Keyword::Haste,
            Keyword::Protection(Color::Black),
            Keyword::Protection(Color::Red),
        ],
        ..creature(
            "Akroma, Angel of Wrath",
            cost(&[generic(5), w(), w(), w()]),
            vec![CreatureType::Angel],
            6,
            6,
        )
    }
}

/// Aven Redeemer — a repeatable 2-point shield.
pub fn aven_redeemer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature(
            "Aven Redeemer",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Aven Warhawk — Amplify 1 over Birds and Soldiers.
pub fn aven_warhawk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: amplify(1, &[CreatureType::Bird, CreatureType::Soldier]),
        ..creature(
            "Aven Warhawk",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Cloudreach Cavalry — a 3/3 flier while you have a Bird.
pub fn cloudreach_cavalry() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "While you control a Bird, this gets +2/+2 and has flying.",
            effect: StaticEffect::PumpSelfIf {
                condition: you_control(R::HasCreatureType(CreatureType::Bird)),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..creature(
            "Cloudreach Cavalry",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Daru Mender — Morph {W}, regenerating on the flip.
pub fn daru_mender() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[w()]))],
        triggered_abilities: vec![on_turn_up(Effect::Regenerate {
            what: target_filtered(R::Creature),
        })],
        ..creature(
            "Daru Mender",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Daru Sanctifier — Morph {1}{W} into enchantment removal.
pub fn daru_sanctifier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), w()]))],
        triggered_abilities: vec![on_turn_up(Effect::Destroy {
            what: target_filtered(R::Enchantment),
        })],
        ..creature(
            "Daru Sanctifier",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            4,
        )
    }
}

/// Daru Stinger — Amplify 1 over Soldiers, then a counter-scaled ping.
pub fn daru_stinger() -> CardDefinition {
    CardDefinition {
        enters_with_counters: amplify(1, &[CreatureType::Soldier]),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
            },
            ..Default::default()
        }],
        ..creature(
            "Daru Stinger",
            cost(&[generic(3), w()]),
            vec![CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Defender of the Order — Morph {W}{W} into a team toughness pump.
pub fn defender_of_the_order() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[w(), w()]))],
        triggered_abilities: vec![on_turn_up(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::ZERO,
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Defender of the Order",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            4,
        )
    }
}

/// Deftblade Elite — Provoke plus a personal fog.
pub fn deftblade_elite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![provoke()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::PreventAllCombatDamageInvolving { target: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Deftblade Elite",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Gempalm Avenger — cycles into a Soldier war cry.
pub fn gempalm_avenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2), w()]))],
        triggered_abilities: vec![on_cycle(Effect::Seq(vec![
            Effect::PumpPT {
                what: all_of(CreatureType::Soldier),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: all_of(CreatureType::Soldier),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Gempalm Avenger",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            5,
        )
    }
}

/// Liege of the Axe — Morph {1}{W}; the flip untaps it.
pub fn liege_of_the_axe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Morph(cost(&[generic(1), w()]))],
        triggered_abilities: vec![on_turn_up(Effect::Untap { what: Selector::This, up_to: None })],
        ..creature(
            "Liege of the Axe",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Lowland Tracker — a first-striking provoker.
pub fn lowland_tracker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![provoke()],
        ..creature(
            "Lowland Tracker",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Starlight Invoker — the white {7} invoker.
pub fn starlight_invoker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), w()]),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
            ..Default::default()
        }],
        ..creature(
            "Starlight Invoker",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Mutant],
            1,
            3,
        )
    }
}

/// Stoic Champion — grows off every cycle in the game.
pub fn stoic_champion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::AnyPlayer),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Stoic Champion",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Swooping Talon — a provoking flier that can ground itself to block.
pub fn swooping_talon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![provoke()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::LoseKeyword { duration: Duration::EndOfTurn,
                what: Selector::This,
                keyword: Keyword::Flying,
            },
            ..Default::default()
        }],
        ..creature(
            "Swooping Talon",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            6,
        )
    }
}

/// Wall of Hope — every point of damage it takes is a point of life.
pub fn wall_of_hope() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..creature("Wall of Hope", cost(&[w()]), vec![CreatureType::Wall], 0, 3)
    }
}

/// Wingbeat Warrior — Morph {2}{W} into a first-strike trick.
pub fn wingbeat_warrior() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(2), w()]))],
        triggered_abilities: vec![on_turn_up(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::FirstStrike,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Wingbeat Warrior",
            cost(&[generic(2), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier, CreatureType::Warrior],
            2,
            1,
        )
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Aven Envoy — a 0/2 flying blocker.
pub fn aven_envoy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature(
            "Aven Envoy",
            cost(&[u()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            0,
            2,
        )
    }
}

/// Cephalid Pathmage — unblockable, and it can hand that off once.
pub fn cephalid_pathmage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Cephalid Pathmage",
            cost(&[generic(2), u()]),
            vec![CreatureType::Octopus, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Covert Operative — a 3/2 that can't be blocked.
pub fn covert_operative() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        ..creature(
            "Covert Operative",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Dreamborn Muse — every upkeep, mill your own hand size.
pub fn dreamborn_muse() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::HandSizeOf(PlayerRef::ActivePlayer),
            },
        }],
        ..creature(
            "Dreamborn Muse",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Echo Tracer — Morph {2}{U} into a bounce.
pub fn echo_tracer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::Move {
            what: target_filtered(R::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..creature(
            "Echo Tracer",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Gempalm Sorcerer — cycles the Wizards into the air.
pub fn gempalm_sorcerer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2), u()]))],
        triggered_abilities: vec![on_cycle(Effect::GrantKeyword {
            what: all_of(CreatureType::Wizard),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Gempalm Sorcerer",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Sorcerer],
            2,
            2,
        )
    }
}

/// Glintwing Invoker — the blue {7} invoker.
pub fn glintwing_invoker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), u()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Glintwing Invoker",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Mutant],
            3,
            3,
        )
    }
}

/// Keeneye Aven — a cycling flier.
pub fn keeneye_aven() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Cycling(cost(&[generic(2)]))],
        ..creature(
            "Keeneye Aven",
            cost(&[generic(3), u()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Merchant of Secrets — a cantrip body.
pub fn merchant_of_secrets() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        ..creature(
            "Merchant of Secrets",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Mistform Ultimus — every creature type at once.
pub fn mistform_ultimus() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Changeling],
        ..creature(
            "Mistform Ultimus",
            cost(&[generic(3), u()]),
            vec![CreatureType::Illusion],
            3,
            3,
        )
    }
}

/// Primoc Escapee — a cycling 4/4 flier.
pub fn primoc_escapee() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Cycling(cost(&[generic(2)]))],
        ..creature(
            "Primoc Escapee",
            cost(&[generic(6), u()]),
            vec![CreatureType::Bird, CreatureType::Beast],
            4,
            4,
        )
    }
}

/// Riptide Director — a Wizard-count draw engine.
pub fn riptide_director() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            tap_cost: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: count_on_battlefield(
                    R::HasCreatureType(CreatureType::Wizard).and(R::ControlledByYou),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Riptide Director",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Voidmage Apprentice — Morph {2}{U}{U} into a counterspell.
pub fn voidmage_apprentice() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), u(), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack),
        })],
        ..creature(
            "Voidmage Apprentice",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Wall of Deceit — a 0/5 that can hide again.
pub fn wall_of_deceit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Morph(cost(&[u()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::TurnFaceDown { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Wall of Deceit", cost(&[generic(1), u()]), vec![CreatureType::Wall], 0, 5)
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Aphetto Exterminator — Morph {3}{B} into a -3/-3.
pub fn aphetto_exterminator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), b()]))],
        triggered_abilities: vec![on_turn_up(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Aphetto Exterminator",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            1,
        )
    }
}

/// Blood Celebrant — any color, one life at a time.
pub fn blood_celebrant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            life_cost: 1,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Blood Celebrant",
            cost(&[b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Dripping Dead — an attacker that eats what it hits.
pub fn dripping_dead() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::DestroyNoRegen { what: Selector::Target(0) },
        }],
        ..creature(
            "Dripping Dead",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Zombie],
            4,
            1,
        )
    }
}

/// Embalmed Brawler — Amplify 1 over Zombies, paid for in life.
pub fn embalmed_brawler() -> CardDefinition {
    CardDefinition {
        enters_with_counters: amplify(1, &[CreatureType::Zombie]),
        triggered_abilities: [EventKind::Attacks, EventKind::Blocks]
            .into_iter()
            .map(|kind| TriggeredAbility {
                event: EventSpec::new(kind, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                },
            })
            .collect(),
        ..creature(
            "Embalmed Brawler",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie],
            2,
            2,
        )
    }
}

/// Gempalm Polluter — cycles into a Zombie-count drain.
pub fn gempalm_polluter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[b(), b()]))],
        triggered_abilities: vec![on_cycle(Effect::LoseLife {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: count_on_battlefield(R::HasCreatureType(CreatureType::Zombie)),
        })],
        ..creature(
            "Gempalm Polluter",
            cost(&[generic(5), b()]),
            vec![CreatureType::Zombie],
            4,
            3,
        )
    }
}

/// Graveborn Muse — a Zombie-count draw that bites.
pub fn graveborn_muse() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: count_on_battlefield(
                        R::HasCreatureType(CreatureType::Zombie).and(R::ControlledByYou),
                    ),
                },
                Effect::LoseLife {
                    who: Selector::You,
                    amount: count_on_battlefield(
                        R::HasCreatureType(CreatureType::Zombie).and(R::ControlledByYou),
                    ),
                },
            ]),
        }],
        ..creature(
            "Graveborn Muse",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Havoc Demon — a 5/5 flier whose death is a sweeper.
pub fn havoc_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(-5),
                toughness: Value::Const(-5),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Havoc Demon",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Demon],
            5,
            5,
        )
    }
}

/// Noxious Ghoul — every Zombie arrival shrinks everything else.
pub fn noxious_ghoul() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Zombie),
                }),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Zombie).negate()),
                ),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Noxious Ghoul",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Zombie],
            3,
            3,
        )
    }
}

/// Skinthinner — Morph {3}{B}{B} into unconditional removal.
pub fn skinthinner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), b(), b()]))],
        triggered_abilities: vec![on_turn_up(Effect::DestroyNoRegen {
            what: target_filtered(R::Creature.and(R::HasColor(Color::Black).negate())),
        })],
        ..creature("Skinthinner", cost(&[generic(1), b()]), vec![CreatureType::Zombie], 2, 1)
    }
}

/// Smokespew Invoker — the black {7} invoker.
pub fn smokespew_invoker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), b()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Smokespew Invoker",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Mutant],
            3,
            1,
        )
    }
}

/// Sootfeather Flock — a Morph {3}{B} flier.
pub fn sootfeather_flock() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(3), b()]))],
        ..creature("Sootfeather Flock", cost(&[generic(4), b()]), vec![CreatureType::Bird], 3, 2)
    }
}

/// Zombie Brute — Amplify 1 over Zombies, with trample.
pub fn zombie_brute() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: amplify(1, &[CreatureType::Zombie]),
        ..creature("Zombie Brute", cost(&[generic(6), b()]), vec![CreatureType::Zombie], 5, 4)
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Clickslither — a Goblin sacrifice outlet that swings for it.
pub fn clickslither() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Goblin), 1)),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Clickslither",
            cost(&[generic(1), r(), r(), r()]),
            vec![CreatureType::Insect],
            3,
            3,
        )
    }
}

/// Goblin Firebug — its exit costs you a land.
pub fn goblin_firebug() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Land,
            },
        }],
        ..creature("Goblin Firebug", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 2, 2)
    }
}

/// Goblin Grappler — a one-drop provoker.
pub fn goblin_grappler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![provoke()],
        ..creature("Goblin Grappler", cost(&[r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Goblin Lookout — feed it a Goblin, the tribe swings bigger.
pub fn goblin_lookout() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Goblin), 1)),
            effect: Effect::PumpPT {
                what: all_of(CreatureType::Goblin),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Goblin Lookout", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 1, 2)
    }
}

/// Goblin Turncoat — a Goblin a turn keeps it alive.
pub fn goblin_turncoat() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Goblin), 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Goblin Turncoat",
            cost(&[generic(1), b()]),
            vec![CreatureType::Goblin, CreatureType::Mercenary],
            2,
            1,
        )
    }
}

/// Goblin Dynamo — a repeatable ping with an X-damage finale.
pub fn goblin_dynamo() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[x(), r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
                ..Default::default()
            },
        ],
        ..creature(
            "Goblin Dynamo",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Mutant],
            4,
            4,
        )
    }
}

/// Frenetic Raptor — the Beasts on the other side can't block.
pub fn frenetic_raptor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Beasts can't block.",
            effect: StaticEffect::GrantKeyword {
                applies_to: all_of(CreatureType::Beast),
                keyword: Keyword::CantBlock,
            },
        }],
        ..creature(
            "Frenetic Raptor",
            cost(&[generic(5), r()]),
            vec![CreatureType::Dinosaur, CreatureType::Beast],
            6,
            6,
        )
    }
}

/// Macetail Hystrodon — a cycling first-strike haste beater.
pub fn macetail_hystrodon() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Haste,
            Keyword::Cycling(cost(&[generic(3)])),
        ],
        ..creature("Macetail Hystrodon", cost(&[generic(6), r()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Rockshard Elemental — Morph {4}{R}{R} into double strike.
pub fn rockshard_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Morph(cost(&[generic(4), r(), r()]))],
        ..creature(
            "Rockshard Elemental",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Elemental],
            4,
            3,
        )
    }
}

/// Skirk Outrider — a Beast in play makes it a 4/4 trampler.
pub fn skirk_outrider() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "While you control a Beast, this gets +2/+2 and has trample.",
            effect: StaticEffect::PumpSelfIf {
                condition: you_control(R::HasCreatureType(CreatureType::Beast)),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..creature("Skirk Outrider", cost(&[generic(3), r()]), vec![CreatureType::Goblin], 2, 2)
    }
}

/// Warbreak Trumpeter — Morph {X}{X}{R} for X Goblins.
pub fn warbreak_trumpeter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[x(), x(), r()]))],
        triggered_abilities: vec![on_turn_up(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: Box::new(goblin_token()),
        })],
        ..creature("Warbreak Trumpeter", cost(&[r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Branchsnap Lorian — a 4/1 trampler that hides behind Morph {G}.
pub fn branchsnap_lorian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Morph(cost(&[g()]))],
        ..creature(
            "Branchsnap Lorian",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Beast],
            4,
            1,
        )
    }
}

/// Brontotherium — a trampling provoker.
pub fn brontotherium() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![provoke()],
        ..creature(
            "Brontotherium",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Beast],
            5,
            3,
        )
    }
}

/// Defiant Elf — a trampling one-drop.
pub fn defiant_elf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Defiant Elf", cost(&[g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Enormous Baloth — a 7/7 for seven.
pub fn enormous_baloth() -> CardDefinition {
    creature("Enormous Baloth", cost(&[generic(6), g()]), vec![CreatureType::Beast], 7, 7)
}

/// Gempalm Strider — cycles into an Elf pump.
pub fn gempalm_strider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2), g(), g()]))],
        triggered_abilities: vec![on_cycle(Effect::PumpPT {
            what: all_of(CreatureType::Elf),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Gempalm Strider", cost(&[generic(1), g()]), vec![CreatureType::Elf], 2, 2)
    }
}

/// Glowering Rogon — Amplify 1 over Beasts.
pub fn glowering_rogon() -> CardDefinition {
    CardDefinition {
        enters_with_counters: amplify(1, &[CreatureType::Beast]),
        ..creature("Glowering Rogon", cost(&[generic(5), g()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Hundroog — a cycling wall of meat.
pub fn hundroog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(3)]))],
        ..creature("Hundroog", cost(&[generic(6), g()]), vec![CreatureType::Beast], 4, 7)
    }
}

/// Krosan Vorine — a provoker nothing can gang up on.
pub fn krosan_vorine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        triggered_abilities: vec![provoke()],
        ..creature(
            "Krosan Vorine",
            cost(&[generic(3), g()]),
            vec![CreatureType::Cat, CreatureType::Beast],
            3,
            2,
        )
    }
}

/// Needleshot Gourna — a 3/6 reach blocker.
pub fn needleshot_gourna() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        ..creature(
            "Needleshot Gourna",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Beast],
            3,
            6,
        )
    }
}

/// Patron of the Wild — Morph {2}{G} into a +3/+3.
pub fn patron_of_the_wild() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), g()]))],
        triggered_abilities: vec![on_turn_up(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(3),
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Patron of the Wild", cost(&[g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Primal Whisperer — it scales with every face-down creature in play.
pub fn primal_whisperer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), g()]))],
        static_abilities: vec![StaticAbility {
            description: "+2/+2 for each face-down creature on the battlefield.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: Value::Times(Box::new(Value::FaceDownCreatures), Box::new(Value::Const(2))),
                toughness: Value::Times(
                    Box::new(Value::FaceDownCreatures),
                    Box::new(Value::Const(2)),
                ),
            },
        }],
        ..creature(
            "Primal Whisperer",
            cost(&[generic(4), g()]),
            vec![CreatureType::Elf, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Stonewood Invoker — the green {7} invoker.
pub fn stonewood_invoker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(5),
                toughness: Value::Const(5),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Stonewood Invoker",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Mutant],
            2,
            2,
        )
    }
}

/// Totem Speaker — Beasts pay you in life.
pub fn totem_speaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Beast),
                }),
            effect: Effect::MayDo {
                description: "Gain 3 life?".into(),
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                }),
            },
        }],
        ..creature(
            "Totem Speaker",
            cost(&[generic(4), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Vexing Beetle — uncounterable, and huge while the board's empty.
pub fn vexing_beetle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        static_abilities: vec![StaticAbility {
            description: "+3/+3 while no opponent controls a creature.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    n: Value::ONE,
                })),
                power: 3,
                toughness: 3,
                keywords: vec![],
            },
        }],
        ..creature("Vexing Beetle", cost(&[generic(4), g()]), vec![CreatureType::Insect], 3, 3)
    }
}

/// Wirewood Channeler — one color, Elf-many.
pub fn wirewood_channeler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(count_on_battlefield(R::HasCreatureType(
                    CreatureType::Elf,
                ))),
            },
            ..Default::default()
        }],
        ..creature(
            "Wirewood Channeler",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            2,
            2,
        )
    }
}

// ── Slivers ─────────────────────────────────────────────────────────────────

/// Blade Sliver — all Slivers get +1/+0.
pub fn blade_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Sliver creatures get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: all_of(CreatureType::Sliver),
                power: 1,
                toughness: 0,
            },
        }],
        ..sliver("Blade Sliver", cost(&[generic(2), r()]), 2, 2)
    }
}

/// Essence Sliver — every point a Sliver deals is life for its controller.
pub fn essence_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Whenever a Sliver deals damage, its controller gains that much life.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::HasCreatureType(CreatureType::Sliver),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
                    effect: Effect::GainLife {
                        who: Selector::You,
                        amount: Value::TriggerEventAmount,
                    },
                }),
            },
        }],
        ..sliver("Essence Sliver", cost(&[generic(3), w()]), 3, 3)
    }
}

/// Root Sliver — Sliver spells can't be countered.
pub fn root_sliver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        static_abilities: vec![StaticAbility {
            description: "Sliver spells can't be countered.",
            effect: StaticEffect::SpellsCantBeCounteredMatching {
                filter: R::HasCreatureType(CreatureType::Sliver),
            },
        }],
        ..sliver("Root Sliver", cost(&[generic(3), g()]), 2, 2)
    }
}

/// Shifting Sliver — Slivers can only be blocked by Slivers.
pub fn shifting_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Slivers can't be blocked except by Slivers.",
            effect: StaticEffect::GrantKeyword {
                applies_to: all_of(CreatureType::Sliver),
                keyword: Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(
                    CreatureType::Sliver,
                ))),
            },
        }],
        ..sliver("Shifting Sliver", cost(&[generic(3), u()]), 2, 2)
    }
}

/// Spectral Sliver — every Sliver gets a firebreathing-style pump.
pub fn spectral_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Slivers have \"{2}: This creature gets +1/+1 until end of turn.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: all_of(CreatureType::Sliver),
                ability: ActivatedAbility {
                    mana_cost: cost(&[generic(2)]),
                    effect: Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..CardDefinition {
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Sliver, CreatureType::Spirit],
                ..Default::default()
            },
            ..sliver("Spectral Sliver", cost(&[generic(2), b()]), 2, 2)
        }
    }
}

/// Synapse Sliver — a Sliver connecting is a card.
pub fn synapse_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Whenever a Sliver deals combat damage to a player, its controller may draw a card.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::HasCreatureType(CreatureType::Sliver),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToPlayer,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::MayDo {
                        description: "Draw a card?".into(),
                        body: Box::new(draw(1)),
                    },
                }),
            },
        }],
        ..sliver("Synapse Sliver", cost(&[generic(4), u()]), 3, 3)
    }
}

/// Toxin Sliver — a Sliver connecting with a creature kills it.
pub fn toxin_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Whenever a Sliver deals combat damage to a creature, destroy that creature.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::HasCreatureType(CreatureType::Sliver),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToCreature,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::DestroyNoRegen { what: Selector::Target(0) },
                }),
            },
        }],
        ..sliver("Toxin Sliver", cost(&[generic(3), b()]), 3, 3)
    }
}

/// Quick Sliver — everyone's Slivers gain flash.
pub fn quick_sliver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "Any player may cast Sliver spells as though they had flash.",
            effect: StaticEffect::AnyPlayerSpellsHaveFlash {
                filter: R::HasCreatureType(CreatureType::Sliver),
            },
        }],
        ..sliver("Quick Sliver", cost(&[generic(1), g()]), 1, 1)
    }
}


// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Bane of the Living — Morph {X}{B}{B} for an X-sized sweeper.
pub fn bane_of_the_living() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[x(), b(), b()]))],
        triggered_abilities: vec![on_turn_up(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(-1))),
            toughness: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(-1))),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Bane of the Living", cost(&[generic(2), b(), b()]), vec![CreatureType::Insect], 4, 3)
    }
}

/// Berserk Murlodont — every Beast grows with the gang that blocks it.
pub fn berserk_murlodont() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Beast),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::BlockersOf(Box::new(Selector::TriggerSource)),
                toughness: Value::BlockersOf(Box::new(Selector::TriggerSource)),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Berserk Murlodont", cost(&[generic(4), g()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Bloodstoke Howler — Morph {6}{R} into a Beast war cry.
pub fn bloodstoke_howler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(6), r()]))],
        triggered_abilities: vec![on_turn_up(Effect::PumpPT {
            what: your_creatures_of(CreatureType::Beast),
            power: Value::Const(3),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Bloodstoke Howler", cost(&[generic(5), r()]), vec![CreatureType::Beast], 3, 4)
    }
}

/// Brood Sliver — every connecting Sliver breeds another.
pub fn brood_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Whenever a Sliver deals combat damage to a player, its controller may create a 1/1 Sliver token.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::HasCreatureType(CreatureType::Sliver),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToPlayer,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::MayDo {
                        description: "Create a 1/1 Sliver token?".into(),
                        body: Box::new(Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Box::new(sliver_token()),
                        }),
                    },
                }),
            },
        }],
        ..sliver("Brood Sliver", cost(&[generic(4), g()]), 3, 3)
    }
}

/// Deathmark Prelate — a Zombie a turn buys unconditional removal.
pub fn deathmark_prelate() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            sorcery_speed: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Zombie), 1)),
            effect: Effect::DestroyNoRegen {
                what: target_filtered(
                    R::Creature.and(R::HasCreatureType(CreatureType::Zombie).negate()),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Deathmark Prelate",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            3,
        )
    }
}

/// Drinker of Sorrow — it hits like a truck and costs you a permanent.
pub fn drinker_of_sorrow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamage, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Permanent,
            },
        }],
        ..creature("Drinker of Sorrow", cost(&[generic(2), b()]), vec![CreatureType::Horror], 5, 3)
    }
}

/// Earthblighter — feed it a Goblin, lose them a land.
pub fn earthblighter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Goblin), 1)),
            effect: Effect::Destroy { what: target_filtered(R::Land) },
            ..Default::default()
        }],
        ..creature(
            "Earthblighter",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Elvish Soultiller — its death reloads a whole tribe into your library.
pub fn elvish_soultiller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::ChooseCreatureTypeThen {
                who: PlayerRef::You,
                then: Box::new(Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::Creature.and(R::IsSourceChosenCreatureType),
                    },
                    to: ZoneDest::Library {
                        who: PlayerRef::You,
                        pos: crate::effect::LibraryPosition::Shuffled,
                    },
                }),
            },
        }],
        ..creature(
            "Elvish Soultiller",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Mutant],
            5,
            4,
        )
    }
}

/// Goblin Clearcutter — a Forest for three red-green mana.
pub fn goblin_clearcutter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Land.and(R::HasLandType(crate::card::LandType::Forest)), 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Red, Color::Green], Value::Const(3)),
            },
            ..Default::default()
        }],
        ..creature("Goblin Clearcutter", cost(&[generic(3), r()]), vec![CreatureType::Goblin], 3, 3)
    }
}

/// Imperial Hellkite — Morph {6}{R}{R} into another Dragon.
pub fn imperial_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(6), r(), r()]))],
        triggered_abilities: vec![on_turn_up(Effect::MayDo {
            description: "Search your library for a Dragon?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Dragon),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Imperial Hellkite",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Infernal Caretaker — Morph {3}{B} to reload every Zombie graveyard.
pub fn infernal_caretaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), b()]))],
        triggered_abilities: vec![on_turn_up(Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::EachPlayer,
                zone: crate::card::Zone::Graveyard,
                filter: R::HasCreatureType(CreatureType::Zombie),
            },
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
        })],
        ..creature(
            "Infernal Caretaker",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Krosan Cloudscraper — a 13/13 with an upkeep tax.
pub fn krosan_cloudscraper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(7), g(), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::You,
                cost: crate::card::WardCost::Mana(cost(&[g(), g()])),
                then: Box::new(Effect::SacrificeSource),
                if_paid: None,
            },
        }],
        ..creature(
            "Krosan Cloudscraper",
            cost(&[generic(7), g(), g(), g()]),
            vec![CreatureType::Beast, CreatureType::Mutant],
            13,
            13,
        )
    }
}

/// Lavaborn Muse — an empty hand across the table is 3 damage a turn.
pub fn lavaborn_muse() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::OpponentControl,
            )
            .with_filter(Predicate::ValueAtMost(
                Value::HandSizeOf(PlayerRef::ActivePlayer),
                Value::Const(2),
            )),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(3),
            },
        }],
        ..creature("Lavaborn Muse", cost(&[generic(3), r()]), vec![CreatureType::Spirit], 3, 3)
    }
}

/// Master of the Veil — Morph {2}{U} to hide someone else's morph again.
pub fn master_of_the_veil() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::MayDo {
            description: "Turn a morph creature face down?".into(),
            body: Box::new(Effect::TurnFaceDown {
                what: target_filtered(R::Creature.and(R::HasMorphAbility)),
            }),
        })],
        ..creature(
            "Master of the Veil",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Mistform Seaswift — a shifty Morph {1}{U} flier.
pub fn mistform_seaswift() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(1), u()]))],
        activated_abilities: vec![become_chosen_type()],
        ..creature("Mistform Seaswift", cost(&[generic(3), u()]), vec![CreatureType::Illusion], 3, 1)
    }
}

/// Mistform Sliver — every Sliver can pass for anything.
pub fn mistform_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Slivers have \"{1}: This becomes the creature type of your choice.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: all_of(CreatureType::Sliver),
                ability: become_chosen_type(),
                condition: None,
            },
        }],
        ..CardDefinition {
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Illusion, CreatureType::Sliver],
                ..Default::default()
            },
            ..sliver("Mistform Sliver", cost(&[generic(1), u()]), 1, 1)
        }
    }
}

/// Mistform Wakecaster — retypes itself, or the whole team.
pub fn mistform_wakecaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            become_chosen_type(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), u()]),
                tap_cost: true,
                effect: Effect::BecomeChosenCreatureType {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    duration: Duration::EndOfTurn,
                    excluded: vec![],
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Mistform Wakecaster",
            cost(&[generic(4), u()]),
            vec![CreatureType::Illusion],
            2,
            3,
        )
    }
}

/// Phage the Untouchable — a win condition that kills you if it cheats in.
pub fn phage_the_untouchable() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::Not(Box::new(Predicate::SourceCastFromOwnersHand))),
                effect: Effect::LoseGame { who: PlayerRef::You },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToCreature,
                    EventScope::SelfSource,
                ),
                effect: Effect::DestroyNoRegen { what: Selector::Target(0) },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::SelfSource,
                ),
                effect: Effect::LoseGame { who: PlayerRef::Triggerer },
            },
        ],
        ..creature(
            "Phage the Untouchable",
            cost(&[generic(3), b(), b(), b(), b()]),
            vec![CreatureType::Avatar, CreatureType::Minion],
            4,
            4,
        )
    }
}

/// Scion of Darkness — connecting steals a body out of their graveyard.
pub fn scion_of_darkness() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Cycling(cost(&[generic(3)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Reanimate a creature from their graveyard?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::InOpponentGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature(
            "Scion of Darkness",
            cost(&[generic(5), b(), b(), b()]),
            vec![CreatureType::Avatar],
            6,
            6,
        )
    }
}

/// Shaleskin Plower — Morph {4}{R} into land destruction.
pub fn shaleskin_plower() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(4), r()]))],
        triggered_abilities: vec![on_turn_up(Effect::Destroy { what: target_filtered(R::Land) })],
        ..creature("Shaleskin Plower", cost(&[generic(3), r()]), vec![CreatureType::Beast], 3, 2)
    }
}

/// Skirk Alarmist — flip someone's morph up early, then take it away.
pub fn skirk_alarmist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::TurnFaceUpFree {
                    what: target_filtered(R::Creature.and(R::FaceDown).and(R::ControlledByYou)),
                    if_cant: None,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Skirk Alarmist",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Sunstrike Legionnaire — a tapper that untaps off every arrival.
pub fn sunstrike_legionnaire() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3))),
            },
            ..Default::default()
        }],
        ..creature(
            "Sunstrike Legionnaire",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Tribal Forcemage — Morph {1}{G} into a tribe-wide alpha strike.
pub fn tribal_forcemage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), g()]))],
        triggered_abilities: vec![on_turn_up(Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::IsSourceChosenCreatureType),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::IsSourceChosenCreatureType),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ])),
        })],
        ..creature(
            "Tribal Forcemage",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Unstable Hulk — Morph {3}{R}{R} for one enormous swing, at a cost.
pub fn unstable_hulk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), r(), r()]))],
        triggered_abilities: vec![on_turn_up(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(6),
                toughness: Value::Const(6),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::SkipTurns { who: PlayerRef::You, count: Value::ONE },
        ]))],
        ..creature(
            "Unstable Hulk",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Mutant],
            2,
            2,
        )
    }
}

/// Warped Researcher — every cycle in the game makes it untouchable.
pub fn warped_researcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Shroud,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature(
            "Warped Researcher",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Mutant],
            3,
            4,
        )
    }
}

/// Weaver of Lies — Morph {4}{U} to hide every other morph on the table.
pub fn weaver_of_lies() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(4), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::TurnFaceDown {
            what: Selector::EachPermanent(
                R::Creature.and(R::HasMorphAbility).and(R::IsSource.negate()),
            ),
        })],
        ..creature("Weaver of Lies", cost(&[generic(5), u(), u()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Willbender — Morph {1}{U} to bend a spell somewhere else.
pub fn willbender() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::ChangeTargetOfAbility {
            what: target_filtered(R::IsSpellOnStack),
        })],
        ..creature(
            "Willbender",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// Caller of the Claw — a flash Elf that pays out for the turn's carnage.
/// (The count is the turn's creature deaths under your control; the printed
/// nontoken rider isn't tracked separately.)
pub fn caller_of_the_claw() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CreaturesDiedThisTurn(PlayerRef::You),
            definition: Box::new(bear_token()),
        })],
        ..creature("Caller of the Claw", cost(&[generic(2), g()]), vec![CreatureType::Elf], 2, 2)
    }
}

/// Celestial Gatekeeper — its death buys back two Birds and/or Clerics.
pub fn celestial_gatekeeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            // The exile is applied after the recursion; the printed order has
            // no rules-visible difference (the Gatekeeper isn't a legal pick).
            effect: Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::PermanentCard
                        .and(R::InYourGraveyard)
                        .and(
                            R::HasCreatureType(CreatureType::Bird)
                                .or(R::HasCreatureType(CreatureType::Cleric)),
                        ),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    }),
                },
                Effect::Move { what: Selector::This, to: ZoneDest::Exile },
            ]),
        }],
        ..creature(
            "Celestial Gatekeeper",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Chromeshell Crab — Morph {4}{U} to trade a creature straight up.
pub fn chromeshell_crab() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(4), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::MayDo {
            description: "Exchange control of two creatures?".into(),
            body: Box::new(Effect::ExchangeControl {
                a: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                b: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            }),
        })],
        ..creature(
            "Chromeshell Crab",
            cost(&[generic(4), u()]),
            vec![CreatureType::Crab, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Corpse Harvester — a creature buys a Zombie and a Swamp.
pub fn corpse_harvester() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Zombie),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land.and(R::HasLandType(crate::card::LandType::Swamp)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Corpse Harvester",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Crookclaw Elder — Birds draw, Wizards grant flight.
pub fn crookclaw_elder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                tap_others_cost: Some((R::HasCreatureType(CreatureType::Bird), 2)),
                effect: draw(1),
                ..Default::default()
            },
            ActivatedAbility {
                tap_others_cost: Some((R::HasCreatureType(CreatureType::Wizard), 2)),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Crookclaw Elder",
            cost(&[generic(5), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Ghastly Remains — Amplify 1, and it buys itself back out of the yard.
pub fn ghastly_remains() -> CardDefinition {
    CardDefinition {
        enters_with_counters: amplify(1, &[CreatureType::Zombie]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            ),
            effect: Effect::MayPay {
                description: "Pay {B}{B}{B} to return Ghastly Remains to your hand?".into(),
                mana_cost: cost(&[b(), b(), b()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                else_: None,
            },
        }],
        ..creature("Ghastly Remains", cost(&[b(), b(), b()]), vec![CreatureType::Zombie], 0, 0)
    }
}

/// Keeper of the Nine Gales — two Birds bounce anything.
pub fn keeper_of_the_nine_gales() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            tap_others_cost: Some((R::HasCreatureType(CreatureType::Bird), 2)),
            effect: Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..creature(
            "Keeper of the Nine Gales",
            cost(&[generic(2), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Magma Sliver — every Sliver can channel the swarm into one attacker.
pub fn magma_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Slivers have \"{T}: Target Sliver gets +X/+0, where X is the number of Slivers.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: all_of(CreatureType::Sliver),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::PumpPT {
                        what: target_filtered(R::HasCreatureType(CreatureType::Sliver)),
                        power: count_on_battlefield(R::HasCreatureType(CreatureType::Sliver)),
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..sliver("Magma Sliver", cost(&[generic(3), r()]), 3, 3)
    }
}

/// Skirk Drill Sergeant — every dead Goblin is a dig for another.
pub fn skirk_drill_sergeant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Goblin),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {2}{R} to dig for a Goblin?".into(),
                mana_cost: cost(&[generic(2), r()]),
                body: Box::new(Effect::RevealTopMayPutOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::PermanentCard.and(R::HasCreatureType(CreatureType::Goblin)),
                    counter: None,
                    extra_types: vec![],
                }),
                else_: None,
            },
        }],
        ..creature(
            "Skirk Drill Sergeant",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin],
            2,
            1,
        )
    }
}

/// Beacon of Destiny — {T}: bounce the next hit from a chosen source onto itself.
pub fn beacon_of_destiny() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: Some(Selector::This),
                whole_turn: false,
                exile_top_per_prevented: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Beacon of Destiny",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            3,
        )
    }
}

/// Dark Supplicant — three Clerics fetch Scion of Darkness from any of three zones.
pub fn dark_supplicant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Cleric), 3)),
            effect: Effect::SearchZones {
                who: PlayerRef::You,
                zones: vec![Zone::Graveyard, Zone::Hand, Zone::Library],
                filter: R::HasName("Scion of Darkness".into()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Dark Supplicant",
            cost(&[b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Dermoplasm — flipping it up swaps it for a morph creature straight from hand.
pub fn dermoplasm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(2), u(), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Creature.and(R::HasMorphAbility),
            count: Value::ONE,
            tapped: false,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
            then: Some(Box::new(Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            })),
        })],
        ..creature(
            "Dermoplasm",
            cost(&[generic(2), u()]),
            vec![CreatureType::Shapeshifter],
            1,
            1,
        )
    }
}

/// Goblin Assassin — every Goblin arrival is a coin flip away from a sacrifice.
pub fn goblin_assassin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Goblin),
                },
            ),
            effect: Effect::EachPlayerFlipsCoin {
                who: PlayerRef::EachPlayer,
                on_heads: Box::new(Effect::Noop),
                on_tails: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::You),
                    count: Value::ONE,
                    filter: R::Creature,
                }),
            },
        }],
        ..creature(
            "Goblin Assassin",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Assassin],
            2,
            2,
        )
    }
}

/// Goblin Goon — a 6/6 that only fights when your board outnumbers theirs.
pub fn goblin_goon() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantAttackUnlessMoreCreaturesThanDefender,
            Keyword::CantBlockUnlessMoreCreaturesThanAttacker,
        ],
        ..creature(
            "Goblin Goon",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Mutant],
            6,
            6,
        )
    }
}

/// Hollow Specter — pay {X} on connect to strip the best of X revealed cards.
pub fn hollow_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayPayX {
                description: "Pay {X} to strip X revealed cards down to one?".into(),
                body: Box::new(Effect::DiscardChosenFromRevealed {
                    from: Selector::Player(PlayerRef::Target(0)),
                    reveal: Value::XFromCost,
                }),
            },
        }],
        ..creature(
            "Hollow Specter",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Specter],
            2,
            2,
        )
    }
}

/// Planar Guide — exiles itself to blink every creature until the end step.
pub fn planar_guide() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::ExileLinked { what: Selector::EachPermanent(R::Creature) },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOfMoved,
                            tapped: false,
                        },
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Planar Guide",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Riptide Mangler — copies a creature's power onto its own base, indefinitely.
pub fn riptide_mangler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::SetBasePower {
                what: Selector::This,
                power: Value::PowerOf(Box::new(target_filtered(R::Creature))),
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..creature("Riptide Mangler", cost(&[generic(1), u()]), vec![CreatureType::Beast], 0, 3)
    }
}

/// Whipgrass Entangler — taxes a creature {1} per Cleric to attack or block.
pub fn whipgrass_entangler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantAttackOrBlockUnlessPayPerPermanent(Box::new(
                    R::HasCreatureType(CreatureType::Cleric),
                )),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Whipgrass Entangler",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            3,
        )
    }
}
