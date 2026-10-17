//! Commander: 2026-10-17 (`yl8zjb`) two-card-gap closers — each pair the last
//! missing cards of an unseated deck. Tests in `tests/recent_b/cmdr_oct17_kn.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, TokenDefinition, Zone, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{enrage, etb, on_attack, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn loot() -> Effect {
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
    ])
}

fn tap_target_defenders_creature() -> Effect {
    Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)) }
}

// ── Bilbo, Luckwearer / Bard the Bowman / Guru Pathik / O'aka ───────────────

/// Old Fat Spider Can't See Me — {2}{U} Saga. I: target creature you control
/// gains hexproof while this Saga remains. II: prevent all damage up to one
/// target creature would deal while it remains (CR 615). III, IV: draw.
pub fn old_fat_spider_cant_see_me() -> CardDefinition {
    let held = |what, keyword| Effect::GrantKeyword { what, keyword, duration: Duration::WhileSourceOnBattlefield };
    let draw = || Effect::Draw { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        name: "Old Fat Spider Can't See Me",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, held(target_filtered(R::Creature.and(R::ControlledByYou)), Keyword::Hexproof)),
            (
                2,
                Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 1,
                    filter: R::Creature,
                    effect: Box::new(held(Selector::Target(0), Keyword::DealsNoDamage)),
                },
            ),
            (3, draw()),
            (4, draw()),
        ],
        ..Default::default()
    }
}

// ── Daredevil, Fearless Fighter ─────────────────────────────────────────────

/// Daredevil's Billy Club — {1}{R} Equipment. ETB: 2 damage to any target and
/// 1 damage to you. Equipped creature gets +1/+0 and has menace. Equip {2}.
pub fn daredevils_billy_club() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            Effect::DealDamage { to: Selector::You, amount: Value::ONE },
        ]))],
        ..equipment(
            "Daredevil's Billy Club",
            cost(&[generic(1), r()]),
            cost(&[generic(2)]),
            EquipBonus { power: 1, keywords: vec![Keyword::Menace], ..Default::default() },
        )
    }
}

/// Elektra, Femme Fatale — {3}{R} legendary 3/2 Human Assassin Hero. ETB: you
/// may have her deal 2 damage to you; when you do, she deals 4 damage to
/// target creature (a reflexive trigger, CR 603.12).
pub fn elektra_femme_fatale() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Elektra deals 2 damage to you: she deals 4 damage to target creature".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage { to: Selector::You, amount: Value::Const(2) },
                Effect::Reflexive {
                    body: Box::new(Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) }),
                },
            ])),
        })],
        ..legend(creature(
            "Elektra, Femme Fatale",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Hero],
            3,
            2,
        ))
    }
}

// ── Neerdiv, Devious Diver ──────────────────────────────────────────────────

/// Merfolk Pupil — {1}{U} 1/1 Merfolk Wizard. ETB: draw, then discard.
/// {1}{U}, exile this card from your graveyard: draw, then discard.
pub fn merfolk_pupil() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(loot())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: loot(),
            ..Default::default()
        }],
        ..creature("Merfolk Pupil", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 1, 1)
    }
}

/// Increasing Confusion — {X}{U} Sorcery. Target player mills X, or twice
/// that many if this spell was cast from a graveyard. Flashback {X}{U}.
pub fn increasing_confusion() -> CardDefinition {
    let mill = |amount| Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount };
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[x(), u()]))],
        ..spell(
            "Increasing Confusion",
            cost(&[x(), u()]),
            CardType::Sorcery,
            Effect::If {
                cond: Predicate::CastFromGraveyard,
                then: Box::new(mill(Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(2))))),
                else_: Box::new(mill(Value::XFromCost)),
            },
        )
    }
}

// ── Zalto, Fire Giant Duke ──────────────────────────────────────────────────

/// Zalto, Fire Giant Duke — {3}{R}{R} legendary 7/3 Giant Barbarian, trample.
/// Whenever Zalto is dealt damage, venture into the dungeon.
pub fn zalto_fire_giant_duke() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![enrage(Effect::Venture)],
        ..legend(creature(
            "Zalto, Fire Giant Duke",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Giant, CreatureType::Barbarian],
            7,
            3,
        ))
    }
}

/// Kick in the Door — {R} Sorcery. A +1/+1 counter on target creature; it
/// gains haste and can't be blocked by Walls this turn. Venture into the
/// dungeon.
pub fn kick_in_the_door() -> CardDefinition {
    spell(
        "Kick in the Door",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            grant(Selector::Target(0), Keyword::Haste),
            grant(Selector::Target(0), Keyword::CantBeBlockedByCreatureType(CreatureType::Wall)),
            Effect::Venture,
        ]),
    )
}

// ── April O'Neil, Kunoichi Trainee ──────────────────────────────────────────

/// S.H.I.E.L.D. Spy Kit — {W} Equipment. Equipped creature gets +1/+1 and has
/// "whenever this creature attacks alone, untap it and scry 1". Equip {1}.
pub fn shield_spy_kit() -> CardDefinition {
    equipment(
        "S.H.I.E.L.D. Spy Kit",
        cost(&[w()]),
        cost(&[generic(1)]),
        EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::AttackingAlone),
                effect: Effect::Seq(vec![
                    Effect::Untap { what: Selector::This, up_to: None },
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                ]),
            }],
            ..Default::default()
        },
    )
}

// ── Venus, Torn Between Worlds ──────────────────────────────────────────────

/// Venus, Torn Between Worlds — {4}{G} legendary 5/5 Mutant Frog Turtle.
/// Whenever she's dealt damage, that many +1/+1 counters on her. Whenever a
/// creature you control with a counter on it deals combat damage to a player,
/// you may pay {U}; if you do, draw a card.
pub fn venus_torn_between_worlds() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            enrage(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::WithAnyCounter) },
                ),
                effect: Effect::MayPay {
                    description: "Pay {U}: draw a card".into(),
                    mana_cost: cost(&[u()]),
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: None,
                },
            },
        ],
        ..legend(creature(
            "Venus, Torn Between Worlds",
            cost(&[generic(4), g()]),
            vec![CreatureType::Mutant, CreatureType::Frog, CreatureType::Turtle],
            5,
            5,
        ))
    }
}

// ── Rhoda, Geist Avenger // Timin, Youthful Geist ───────────────────────────

/// Sentinel of the Eternal Watch — {5}{W} 4/6 Giant Soldier, vigilance. At the
/// beginning of combat on each opponent's turn, tap target creature that
/// player controls.
pub fn sentinel_of_the_eternal_watch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::OpponentControl),
            effect: Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByActivePlayer)) },
        }],
        ..creature(
            "Sentinel of the Eternal Watch",
            cost(&[generic(5), w()]),
            vec![CreatureType::Giant, CreatureType::Soldier],
            4,
            6,
        )
    }
}

/// Distracting Geist // Clever Distraction — {2}{W} 2/1 Spirit. Whenever it
/// attacks, tap target creature defending player controls. Disturb {4}{W}: an
/// Aura granting the enchanted creature that same attack trigger.
pub fn distracting_geist() -> CardDefinition {
    let tapper = || TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: tap_target_defenders_creature(),
    };
    let clever = CardDefinition {
        name: "Clever Distraction",
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { triggered_abilities: vec![tapper()], ..Default::default() }),
        color_indicator: vec![Color::White],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Disturb(cost(&[generic(4), w()]))],
        triggered_abilities: vec![on_attack(tap_target_defenders_creature())],
        back_face: Some(Box::new(clever)),
        ..creature("Distracting Geist", cost(&[generic(2), w()]), vec![CreatureType::Spirit], 2, 1)
    }
}

// ── Jurin, Leading the Charge ───────────────────────────────────────────────

/// Jurin, Leading the Charge — {4}{R}{R} legendary 4/6 Human Barbarian,
/// haste, must be blocked if able. Whenever Jurin attacks, each creature you
/// control attacking a player gets +1/+0 for each creature *that* player
/// controls (CR 506.2: each attacker's own defender).
pub fn jurin_leading_the_charge() -> CardDefinition {
    let theirs = Value::count(Selector::ControlledBy {
        who: PlayerRef::DefendingPlayerOf(Box::new(Selector::TriggerSource)),
        filter: R::Creature,
    });
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::MustBeBlocked],
        triggered_abilities: vec![on_attack(Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::IsAttackingOpponentPlayer)),
            body: Box::new(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: theirs,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
        })],
        ..legend(creature(
            "Jurin, Leading the Charge",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            4,
            6,
        ))
    }
}

// ── White Tiger, Amulet Keeper ──────────────────────────────────────────────

/// White Tiger, Amulet Keeper — {1}{G} legendary 3/1 Human Hero. {3}{G}, exile
/// this card from your graveyard: draw a card; you may put a land card from
/// your hand onto the battlefield.
pub fn white_tiger_amulet_keeper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Land,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
            ]),
            ..Default::default()
        }],
        ..legend(creature(
            "White Tiger, Amulet Keeper",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Hero],
            3,
            1,
        ))
    }
}

/// Hollow Trees — land, enters tapped, may stay tapped (CR 502.3). Upkeep, if
/// tapped: a storage counter. {T}, remove any number of storage counters: {G}
/// for each.
pub fn hollow_trees() -> CardDefinition {
    CardDefinition {
        name: "Hollow Trees",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::MayChooseNotToUntap],
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Tapped }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Storage, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_x: Some(CounterType::Storage),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![Color::Green], Value::XFromCost) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Behold (CR 701.4) ───────────────────────────────────────────────────────

/// Hulk's Thunderclap — {1}{G} Sorcery. You may behold a Gamma creature. Target
/// creature you control deals damage equal to its power to another target
/// creature; if the behold was paid, destroy target noncreature artifact or
/// noncreature enchantment (that slot exists only then, CR 601.2c).
pub fn hulks_thunderclap() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::Behold {
            filter: R::Creature.and(R::HasCreatureType(CreatureType::Gamma)),
        }),
        ..spell(
            "Hulk's Thunderclap",
            cost(&[generic(1), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::DealDamageEqualToPower {
                    source: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                    target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::OtherThanTargetSlot(0)) },
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Destroy {
                        what: Selector::TargetFiltered {
                            slot: 2,
                            filter: R::Artifact.or(R::Enchantment).and(R::Creature.negate()),
                        },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Hyperion's Atomic Vision — {1}{W} Instant. You may behold a Hero. Destroy
/// target tapped creature; if a Hero was beheld, scry 2.
pub fn hyperions_atomic_vision() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::Behold { filter: R::HasCreatureType(CreatureType::Hero) }),
        ..spell(
            "Hyperion's Atomic Vision",
            cost(&[generic(1), w()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

fn behold_or_pay(kind: CreatureType, def: CardDefinition) -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::BeholdOrPay { filter: R::HasCreatureType(kind), pay: 2 }],
        ..def
    }
}

/// Lys Alana Dignitary — {1}{G} 2/3 Elf Advisor. Behold an Elf or pay {2}.
/// {T}: {G}{G}, only with an Elf card in your graveyard.
pub fn lys_alana_dignitary() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::HasCreatureType(CreatureType::Elf),
                },
                n: Value::ONE,
            }),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Green]) },
            ..Default::default()
        }],
        ..behold_or_pay(
            CreatureType::Elf,
            creature("Lys Alana Dignitary", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Advisor], 2, 3),
        )
    }
}

/// Kinsbaile Aspirant — {W} 2/1 Kithkin Citizen. Behold a Kithkin or pay {2}.
/// Whenever another creature you control enters, +1/+1 until end of turn.
pub fn kinsbaile_aspirant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..behold_or_pay(
            CreatureType::Kithkin,
            creature("Kinsbaile Aspirant", cost(&[w()]), vec![CreatureType::Kithkin, CreatureType::Citizen], 2, 1),
        )
    }
}

/// Mudbutton Cursetosser — {B} 2/1 Goblin Warlock. Behold a Goblin or pay {2}.
/// Can't block. Dies: destroy target creature an opponent controls with power
/// 2 or less.
pub fn mudbutton_cursetosser() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![on_dies(Effect::Destroy {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::PowerAtMost(2))),
        })],
        ..behold_or_pay(
            CreatureType::Goblin,
            creature("Mudbutton Cursetosser", cost(&[b()]), vec![CreatureType::Goblin, CreatureType::Warlock], 2, 1),
        )
    }
}

/// Silvergill Mentor — {1}{U} 2/1 Merfolk Wizard. Behold a Merfolk or pay {2}.
/// ETB: a 1/1 white and blue Merfolk token.
pub fn silvergill_mentor() -> CardDefinition {
    let merfolk = std::sync::Arc::new(TokenDefinition {
        name: "Merfolk".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: merfolk })],
        ..behold_or_pay(
            CreatureType::Merfolk,
            creature("Silvergill Mentor", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 1),
        )
    }
}
