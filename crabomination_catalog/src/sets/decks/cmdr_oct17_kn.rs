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

// ── Doc Aurlock, Grizzled Genius / Meneldor, Swift Savior ───────────────────

/// Extraordinary Journey — {X}{X}{U}{U} Enchantment. ETB: exile up to X target
/// creatures; each owner may play theirs while it stays exiled. Whenever one
/// or more nontoken creatures enter from exile or cast from exile, draw a card
/// (once each turn).
pub fn extraordinary_journey() -> CardDefinition {
    CardDefinition {
        name: "Extraordinary Journey",
        cost: cost(&[x(), x(), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CapTargetsAt {
                amount: Value::XFromCost,
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 10,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Move { what: Selector::Target(0), to: crate::effect::ZoneDest::Exile },
                        Effect::GrantMayPlay {
                            what: Selector::LastMoved,
                            duration: crate::card::MayPlayDuration::WhileExiled,
                            to_owner: true,
                            exile_after: false,
                            pay_own_cost: true,
                            any_color: false,
                        },
                    ])),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::NotToken).and(R::EnteredFromExileThisTurn.or(R::WasCastFromExile)),
                    })
                    .once_per_turn(),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

// ── Leonardo, Leader in Blue ────────────────────────────────────────────────

/// Leonardo, Leader in Blue — {W} legendary 2/1 Mutant Ninja Turtle. Sneak
/// {3}{W}{W}; if his sneak cost was paid, creatures you control get +2/+0 until
/// end of turn. {1}{W}: first strike until end of turn.
pub fn leonardo_leader_in_blue() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            marks_kicked: true,
            ..crate::effect::shortcut::sneak(cost(&[generic(3), w(), w()]))
        }),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: grant(Selector::This, Keyword::FirstStrike),
            ..Default::default()
        }],
        ..legend(creature(
            "Leonardo, Leader in Blue",
            cost(&[w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            2,
            1,
        ))
    }
}

/// Prehistoric Pet — {W} 1/2 Dinosaur Ninja; can't be blocked by creatures with
/// greater power (skulk). {1}{W}, {T}: return another target creature you
/// control to its owner's hand; only during your turn.
pub fn prehistoric_pet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Skulk],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature("Prehistoric Pet", cost(&[w()]), vec![CreatureType::Dinosaur, CreatureType::Ninja], 1, 2)
    }
}

// ── Torgal / Jandor / Tolsimir ──────────────────────────────────────────────

/// Bejeweled Warg — {1}{G} 3/2 Wolf, trample. Combat damage to a player: a
/// +1/+1 counter on target Wolf you control, or a Treasure.
pub fn bejeweled_warg() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter {
                    what: target_filtered(R::HasCreatureType(CreatureType::Wolf).and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: std::sync::Arc::new(crate::game::effects::treasure_token()),
                },
            ]),
        }],
        ..creature("Bejeweled Warg", cost(&[generic(1), g()]), vec![CreatureType::Wolf], 3, 2)
    }
}

// ── Higure, the Still Wind ──────────────────────────────────────────────────

/// Futurist Operative — {3}{U} 3/4 Human Ninja. While tapped it's a Human
/// Citizen with base P/T 1/1 and can't be blocked. {2}{U}: untap it.
pub fn futurist_operative() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect as SE};
    let tapped = || Predicate::EntityMatches { what: Selector::This, filter: R::Tapped };
    let while_tapped = |description, inner| StaticAbility {
        description,
        effect: SE::WhileCondition { condition: tapped(), inner: Box::new(inner) },
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as this creature is tapped, its base power and toughness are 1/1.",
                effect: SE::SetBasePtIf { condition: tapped(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "As long as this creature is tapped, it can't be blocked.",
                effect: SE::SelfHasKeywordWhilePredicate { keyword: Keyword::Unblockable, condition: tapped() },
            },
            while_tapped(
                "As long as this creature is tapped, it's a Human Citizen.",
                SE::MatchingLoseAllCreatureTypes { applies_to: Selector::This },
            ),
            while_tapped(
                "As long as this creature is tapped, it's a Human Citizen.",
                SE::AddCreatureTypeToMatching { applies_to: Selector::This, creature_type: CreatureType::Human },
            ),
            while_tapped(
                "As long as this creature is tapped, it's a Human Citizen.",
                SE::AddCreatureTypeToMatching { applies_to: Selector::This, creature_type: CreatureType::Citizen },
            ),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..creature("Futurist Operative", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Ninja], 3, 4)
    }
}

// ── Unesh / Zhuge Jin / The Ur-Sphinx ───────────────────────────────────────

/// Sphinx Ambassador — {5}{U}{U} 5/5 Sphinx, flying. Combat damage to a player:
/// search their library for a card; they name a card; a creature card without
/// that name may enter under your control; they shuffle.
pub fn sphinx_ambassador() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            // Combat binds the damaged player to slot 0.
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::SearchTheirLibraryAgainstNamedCard { who: PlayerRef::Target(0) },
        }],
        ..creature("Sphinx Ambassador", cost(&[generic(5), u(), u()]), vec![CreatureType::Sphinx], 5, 5)
    }
}

// ── Lightning, Security Sergeant / Dragon's Smile / Magik ───────────────────

/// Dragon's Smile — {5}{R}{R} legendary 4/5 Vehicle, flying, crew 3. Once during
/// each of your turns, a spell you cast from exile may cost {0} (CR 118.9).
/// Combat damage to a player: they exile that many cards from the top of their
/// library; you may play them this turn.
pub fn dragons_smile() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect as SE};
    CardDefinition {
        name: "Dragon's Smile",
        cost: cost(&[generic(5), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Crew(3)],
        static_abilities: vec![StaticAbility {
            description: "Once during each of your turns, you may pay {0} rather than pay the mana cost for a spell you cast from exile.",
            effect: SE::WhileYourTurn { inner: Box::new(SE::FreeExileCastOncePerTurn) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::Target(0),
                count: Value::TriggerEventAmount,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

// ── Olinda the Oblivious ────────────────────────────────────────────────────

/// Olinda the Oblivious — {3}{B} legendary 3/3 Skunk Rogue. Whenever she enters
/// or attacks, for each opponent, an odor counter on up to one target creature
/// that player controls without one; each has "at the beginning of your upkeep,
/// you lose 2 life" for as long as it has an odor counter (CR 611.2c).
pub fn olinda_the_oblivious() -> CardDefinition {
    let stink = || Effect::ForEachOpponentTarget {
        body: Box::new(Effect::ApplyToTargets {
            max_targets: 15,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByOpponent).and(R::WithCounter(CounterType::Odor).negate()),
            effect: Box::new(Effect::Seq(vec![
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Odor, amount: Value::ONE },
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                        effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                    }),
                    duration: Duration::WhileHasCounter(CounterType::Odor),
                },
            ])),
        }),
    };
    CardDefinition {
        triggered_abilities: vec![etb(stink()), on_attack(stink())],
        ..legend(creature("Olinda the Oblivious", cost(&[generic(3), b()]), vec![CreatureType::Skunk, CreatureType::Rogue], 3, 3))
    }
}

// ── Thassa, God of the Sea / Slinn Voda ─────────────────────────────────────

/// Reservoir Kraken — {2}{U}{U} 6/6 Kraken, trample, ward {2}. At the beginning
/// of each combat, if it's untapped, any opponent may tap an untapped creature
/// they control; if one does, tap the Kraken and create a 1/1 blue Fish with
/// "this token can't be blocked".
pub fn reservoir_kraken() -> CardDefinition {
    let fish = std::sync::Arc::new(TokenDefinition {
        name: "Fish".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        keywords: vec![Keyword::Unblockable],
        ..Default::default()
    });
    let untapped = || R::Creature.and(R::Tapped.negate());
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Tapped.negate() }),
            effect: Effect::AnyPlayerMayAccept {
                who: PlayerRef::EachOpponent,
                prompt: "Tap an untapped creature you control to tap Reservoir Kraken (its controller gets a Fish)?".into(),
                accepted: Box::new(Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::ControlledBy { who: PlayerRef::AcceptingPlayer, filter: untapped() },
                        n: Value::ONE,
                    },
                    then: Box::new(Effect::Seq(vec![
                        Effect::PlayerTapsUntapped { who: PlayerRef::AcceptingPlayer, filter: untapped(), amount: Value::ONE },
                        Effect::Tap { what: Selector::This },
                        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: fish },
                    ])),
                    else_: Box::new(Effect::Noop),
                }),
                otherwise: Box::new(Effect::Noop),
            },
        }],
        ..creature("Reservoir Kraken", cost(&[generic(2), u(), u()]), vec![CreatureType::Kraken], 6, 6)
    }
}

// ── The Neutrinos ───────────────────────────────────────────────────────────

/// The Neutrinos — {2}{R}{W} legendary 2/4 Elf Rebel, flying. Alliance: another
/// creature of yours entering gives +1/+0 until end of turn. Whenever they
/// attack, exile up to one target creature you own, then return it under your
/// control tapped and attacking (CR 508.4).
pub fn the_neutrinos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            },
            on_attack(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::OwnedByYou),
                // "You own" makes its owner you, so the owner's return is
                // "under your control"; the handle survives the blink, and
                // joining combat taps it (CR 508.4).
                effect: Box::new(Effect::Seq(vec![
                    Effect::ExileAndReturnToOwner { what: Selector::Target(0) },
                    Effect::JoinCombatAttacking { what: Selector::Target(0) },
                ])),
            }),
        ],
        ..legend(creature("The Neutrinos", cost(&[generic(2), r(), w()]), vec![CreatureType::Elf, CreatureType::Rebel], 2, 4))
    }
}

/// Master of the Wild Hunt — {2}{G}{G} 3/3 Human Shaman. Upkeep: a 2/2 green
/// Wolf. {T}: tap all your untapped Wolves; each deals damage equal to its power
/// to target creature, which deals its power back divided among them by its
/// controller.
pub fn master_of_the_wild_hunt() -> CardDefinition {
    let wolf = std::sync::Arc::new(TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: wolf },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TapMatchingDealPowerAndTakeDivided {
                filter: R::HasCreatureType(CreatureType::Wolf),
                target: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature("Master of the Wild Hunt", cost(&[generic(2), g(), g()]), vec![CreatureType::Human, CreatureType::Shaman], 3, 3)
    }
}

// ── Falcon and Redwing ──────────────────────────────────────────────────────

/// Winnowing — {4}{W}{W} Sorcery, convoke. For each player you choose a
/// creature they control; each player sacrifices every other creature that
/// shares no creature type with it.
pub fn winnowing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..spell("Winnowing", cost(&[generic(4), w(), w()]), CardType::Sorcery, Effect::EachPlayerSacrificesUnlikeChosen)
    }
}

// ── Scarlet Spider, Ben Reilly / Miles Morales ──────────────────────────────

/// Scarlet Spider, Ben Reilly — {1}{R}{G} legendary 4/3 Spider Human Hero,
/// trample. Web-slinging {R}{G} (CR 702.188); cast that way he enters with X
/// +1/+1 counters, X the returned creature's mana value (CR 614.12).
pub fn scarlet_spider_ben_reilly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[r(), g()]),
            return_to_hand: Some((R::Creature.and(R::Tapped), 1)),
            returned_mana_value_is_x: true,
            ..Default::default()
        }),
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        ..legend(creature(
            "Scarlet Spider, Ben Reilly",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            4,
            3,
        ))
    }
}

// ── Darigaaz Reincarnated ───────────────────────────────────────────────────

/// Darigaaz Reincarnated — {4}{B}{R}{G} legendary 7/7 Dragon, flying, trample,
/// haste. If it would die, it's exiled with three egg counters instead. At the
/// beginning of your upkeep, if it's exiled with an egg counter, remove one;
/// then with none left, it returns to the battlefield.
pub fn darigaaz_reincarnated() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect as SE};
    let eggs = || Predicate::EntityMatches { what: Selector::This, filter: R::InExile.and(R::WithCounter(CounterType::Egg)) };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "If Darigaaz would die, instead exile it with three egg counters on it.",
            effect: SE::SelfDiesToExileWithCounters { counter: CounterType::Egg, count: 3 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                zone: crate::effect::TriggerZone::WhileExiled,
                ..EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(eggs())
            },
            effect: Effect::If {
                cond: eggs(),
                then: Box::new(Effect::Seq(vec![
                    Effect::RemoveCounter { what: Selector::This, kind: CounterType::Egg, amount: Value::ONE },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::This,
                            filter: R::InExile.and(R::WithCounter(CounterType::Egg).negate()),
                        },
                        then: Box::new(Effect::Move {
                            what: Selector::This,
                            to: crate::effect::ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..legend(creature(
            "Darigaaz Reincarnated",
            cost(&[generic(4), b(), r(), g()]),
            vec![CreatureType::Dragon],
            7,
            7,
        ))
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
