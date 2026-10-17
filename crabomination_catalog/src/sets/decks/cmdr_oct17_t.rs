//! Commander: 2026-10-17 (`3chree`) closers, fourth batch — one-deck closers
//! from the EDHREC long tail. Tests in `tests/recent_b/cmdr_oct17_t.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, Adventure, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, g, generic, hybrid, r, u, w, x};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// Wayward Guide-Beast — {R} 2/2 Beast, trample, haste. Combat damage to a
/// player: return a land you control to its owner's hand (chosen, untargeted).
pub fn wayward_guide_beast() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MoveChosen {
                from: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                filter: None,
                count: Value::ONE,
                up_to: false,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature("Wayward Guide-Beast", cost(&[r()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Riftsweeper — {1}{G} 2/2 Elf Shaman. ETB: target face-up exiled card's
/// owner shuffles it into their library.
pub fn riftsweeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::InExile.and(R::Not(Box::new(R::FaceDown)))),
            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Shuffled },
        })],
        ..creature("Riftsweeper", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Shaman], 2, 2)
    }
}

/// Icy Reception — {1}{U} instant. Choose one — counter target creature or
/// legendary spell unless its controller pays {3}; or target creature gets -5/-0.
pub fn icy_reception() -> CardDefinition {
    spell(
        "Icy Reception",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack.and(R::Creature.or(R::HasSupertype(Supertype::Legendary)))),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
                if_paid: None,
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-5),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Punishing Punch — {2}{G} instant; {2} less with two+ creature cards in your
/// graveyard. Your creature deals twice its power to an opponent's creature
/// (the creature is the source: lifelink/deathtouch apply).
pub fn punishing_punch() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if there are two or more creature cards in your graveyard.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::HasCardType(CardType::Creature) },
                    Value::Const(2),
                ),
                amount: 2,
            },
        }],
        ..spell(
            "Punishing Punch",
            cost(&[generic(2), g()]),
            CardType::Instant,
            Effect::DealDamageFrom {
                source: target_filtered(R::Creature.and(R::ControlledByYou)),
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                amount: Value::Times(
                    Box::new(Value::PowerOf(Box::new(Selector::Target(0)))),
                    Box::new(Value::Const(2)),
                ),
            },
        )
    }
}

/// Compel Brutality — {1}{G} instant. Choose one — your creature deals damage
/// equal to its power, or your planeswalker damage equal to its loyalty, to
/// target creature or planeswalker an opponent controls.
pub fn compel_brutality() -> CardDefinition {
    let victim = || Selector::TargetFiltered {
        slot: 1,
        filter: (R::Creature.or(R::Planeswalker)).and(R::ControlledByOpponent),
    };
    spell(
        "Compel Brutality",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::DealDamageFrom {
                source: target_filtered(R::Creature.and(R::ControlledByYou)),
                to: victim(),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            Effect::DealDamageFrom {
                source: target_filtered(R::Planeswalker.and(R::ControlledByYou)),
                to: victim(),
                amount: Value::LoyaltyOf(Box::new(Selector::Target(0))),
            },
        ]),
    )
}

/// Grafted Growth — {2}{G} Aura, enchant land. ETB: +1/+1 counter on target
/// creature or Vehicle you control. Enchanted land has "{T}: Add two mana of
/// any one color." (New Horizons' shape.)
pub fn grafted_growth() -> CardDefinition {
    CardDefinition {
        name: "Grafted Growth",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                (R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))).and(R::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Add two mana of any one color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: crate::card::ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(2)) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Recursive Recruitment — {2}{U}{B} sorcery. Two 2/2 colorless Wizard Soldier
/// "Cadet" tokens; cast from a graveyard, a +1/+1 counter on each per three
/// cards in your graveyard. Flashback {6}{U}{B}.
pub fn recursive_recruitment() -> CardDefinition {
    let cadet = Arc::new(TokenDefinition {
        name: "Cadet".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(6), u(), b()]))],
        ..spell(
            "Recursive Recruitment",
            cost(&[generic(2), u(), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: cadet },
                Effect::If {
                    cond: Predicate::CastFromGraveyard,
                    then: Box::new(Effect::AddCounter {
                        what: Selector::LastCreatedTokens,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::DivDown(Box::new(Value::GraveyardSizeOf(PlayerRef::You)), 3),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Doublecast — {R}{R} sorcery. When you next cast an instant or sorcery
/// spell this turn, copy it; you may choose new targets.
pub fn doublecast() -> CardDefinition {
    spell(
        "Doublecast",
        cost(&[r(), r()]),
        CardType::Sorcery,
        Effect::OnYourNextInstantSorceryThisTurn {
            body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
        },
    )
}

/// Vizier of Deferment — {2}{W} 2/2 Human Cleric, flash. ETB: you may exile
/// target creature if it attacked or blocked this turn (checked on
/// resolution); it returns under its owner's control at the next end step.
pub fn vizier_of_deferment() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile that creature until the next end step?".into(),
            body: Box::new(Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::AttackedThisTurn.or(R::BlockedThisTurn),
                },
                then: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                    what: target_filtered(R::Creature),
                    tapped: false,
                }),
                else_: Box::new(Effect::Noop),
            }),
        })],
        ..creature("Vizier of Deferment", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 2)
    }
}

/// Catharsis — {4}{R/W}{R/W} 3/4 Elemental Incarnation, evoke {R/W}{R/W}. ETB
/// if {W}{W} spent: two 1/1 green-white Kithkin; if {R}{R} spent: your
/// creatures get +1/+1 and haste until EOT. (Vibrance's shape.)
pub fn catharsis() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    let spent = |color, effect| TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
            .with_filter(Predicate::SourceCastWithColorSpent { color, at_least: 2 }),
        effect,
    };
    let kithkin = Arc::new(TokenDefinition {
        name: "Kithkin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kithkin], ..Default::default() },
        ..Default::default()
    });
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[rw(), rw()]),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        triggered_abilities: vec![
            spent(Color::White, Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: kithkin }),
            spent(
                Color::Red,
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: yours(),
                        power: Value::Const(1),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: yours(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
            ),
        ],
        ..creature(
            "Catharsis",
            cost(&[generic(4), rw(), rw()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            3,
            4,
        )
    }
}

/// Siren Lookout — {2}{U} 1/2 Siren Pirate, flying. ETB: it explores.
pub fn siren_lookout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Explore { who: Selector::This })],
        ..creature("Siren Lookout", cost(&[generic(2), u()]), vec![CreatureType::Siren, CreatureType::Pirate], 1, 2)
    }
}

/// Enter the Unknown — {G} sorcery. Target creature you control explores; you
/// may play an additional land this turn.
pub fn enter_the_unknown() -> CardDefinition {
    spell(
        "Enter the Unknown",
        cost(&[g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Explore { who: target_filtered(R::Creature.and(R::ControlledByYou)) },
            Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE },
        ]),
    )
}

/// Velvetwing Butterflies // Gaze in Wonder — {2}{W} 2/2 flying Insect.
/// Adventure {1}{W} instant: tap one or two target creatures.
pub fn velvetwing_butterflies() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        adventure: Some(Box::new(Adventure {
            name: "Gaze in Wonder",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::OptionalTargets {
                min: 1,
                body: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
                    Effect::Tap { what: Selector::TargetFiltered { slot: 1, filter: R::Creature } },
                ])),
            },
        })),
        ..creature("Velvetwing Butterflies", cost(&[generic(2), w()]), vec![CreatureType::Insect], 2, 2)
    }
}

/// Ettercap // Web Shot — {4}{G} 2/5 reach Spider Beast. Adventure {2}{G}
/// instant: destroy target creature with flying.
pub fn ettercap() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        adventure: Some(Box::new(Adventure {
            name: "Web Shot",
            cost: cost(&[generic(2), g()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))) },
        })),
        ..creature("Ettercap", cost(&[generic(4), g()]), vec![CreatureType::Spider, CreatureType::Beast], 2, 5)
    }
}

/// Gloom Ripper — {3}{B}{B} 4/4 Elf Assassin. ETB: target creature you control
/// gets +X/+0 and up to one target creature an opponent controls gets -0/-X
/// until end of turn, X = Elves you control + Elf cards in your graveyard.
pub fn gloom_ripper() -> CardDefinition {
    let elf = || R::HasCreatureType(CreatureType::Elf);
    let n = || {
        Value::Sum(vec![
            Value::count(Selector::EachPermanent(elf().and(R::ControlledByYou))),
            Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: elf() },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                    power: n(),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                    power: Value::Const(0),
                    toughness: Value::Negate(Box::new(n())),
                    duration: Duration::EndOfTurn,
                },
            ])),
        })],
        ..creature("Gloom Ripper", cost(&[generic(3), b(), b()]), vec![CreatureType::Elf, CreatureType::Assassin], 4, 4)
    }
}

/// Ravenous Tyrannosaurus — {4}{R}{G} 6/6 Dinosaur, devour 3. Attacks: damage
/// equal to its power to up to one other target creature, excess to that
/// creature's controller.
pub fn ravenous_tyrannosaurus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devour(3)],
        triggered_abilities: vec![on_attack(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::DealDamageExcessToController {
                to: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::OtherThanSource) },
                amount: Value::PowerOf(Box::new(Selector::This)),
            }),
        })],
        ..creature("Ravenous Tyrannosaurus", cost(&[generic(4), r(), g()]), vec![CreatureType::Dinosaur], 6, 6)
    }
}

/// Knight of the Holy Nimbus — {W}{W} 2/2 flanking. Destruction regenerates it
/// instead. {2}: it can't be regenerated this turn; only opponents may activate.
pub fn knight_of_the_holy_nimbus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        static_abilities: vec![StaticAbility {
            description: "If this creature would be destroyed, regenerate it.",
            effect: StaticEffect::RegeneratesInsteadOfDestroyed,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::CantBeRegeneratedThisTurn { what: Selector::This },
            opponents_only: true,
            ..Default::default()
        }],
        ..creature(
            "Knight of the Holy Nimbus",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Kenku Artificer — {2}{U} 1/1 Bird Artificer. ETB: three +1/+1 counters on up
/// to one target noncreature artifact; it becomes a 0/0 Homunculus artifact
/// creature with flying.
pub fn kenku_artificer() -> CardDefinition {
    let it = || Selector::TargetFiltered { slot: 0, filter: R::Artifact.and(R::Noncreature) };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: it(),
                    power: Value::Const(0),
                    toughness: Value::Const(0),
                    creature_types: vec![CreatureType::Homunculus],
                    keywords: vec![Keyword::Flying],
                    duration: Duration::Permanent,
                },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(3) },
            ])),
        })],
        ..creature("Kenku Artificer", cost(&[generic(2), u()]), vec![CreatureType::Bird, CreatureType::Artificer], 1, 1)
    }
}

/// Deceit — {4}{U/B}{U/B} 5/5 Elemental Incarnation, evoke {U/B}{U/B}. ETB if
/// {U}{U} spent: bounce up to one other nonland permanent; if {B}{B} spent:
/// target opponent reveals, you pick a nonland card, they discard it.
pub fn deceit() -> CardDefinition {
    let ub = || hybrid(Color::Blue, Color::Black);
    let spent = |color, effect| TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
            .with_filter(Predicate::SourceCastWithColorSpent { color, at_least: 2 }),
        effect,
    };
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[ub(), ub()]),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        triggered_abilities: vec![
            spent(
                Color::Blue,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Move {
                        what: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::Permanent.and(R::Nonland).and(R::OtherThanSource),
                        },
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    }),
                },
            ),
            spent(
                Color::Black,
                Effect::DiscardChosen {
                    from: target_filtered(R::Player.and(R::ControlledByOpponent)),
                    count: Value::ONE,
                    filter: R::Nonland,
                },
            ),
        ],
        ..creature("Deceit", cost(&[generic(4), ub(), ub()]), vec![CreatureType::Elemental, CreatureType::Incarnation], 5, 5)
    }
}

/// Titan's Revenge — {X}{R}{R} sorcery. X damage to any target; clash, and a
/// win returns it to its owner's hand.
pub fn titans_revenge() -> CardDefinition {
    spell(
        "Titan's Revenge",
        cost(&[x(), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
            Effect::ClashWithOpponent { on_win: Box::new(Effect::ReturnResolvingSpellToHand), on_lose: None, with: None },
        ]),
    )
}

/// Theorix Charm — {U}{B} instant. Choose one — counter target noncreature
/// spell unless its controller pays {2}; target creature gets -2/-2 until end
/// of turn; or mill three, then draw a card.
pub fn theorix_charm() -> CardDefinition {
    spell(
        "Theorix Charm",
        cost(&[u(), b()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)),
                mana_cost: cost(&[generic(2)]),
                exile: false,
                extra_generic: None,
                if_paid: None,
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(3) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        ]),
    )
}

/// Seal from Existence — {1}{W}{W} enchantment, ward {3}. ETB: exile target
/// nonland permanent an opponent controls until this leaves the battlefield.
pub fn seal_from_existence() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::generic(3))],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..spell("Seal from Existence", cost(&[generic(1), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Winterthorn Blessing — {G}{U} sorcery. +1/+1 counter on up to one target
/// creature you control; tap up to one target creature you don't control, and
/// it doesn't untap during its controller's next untap step. Flashback {1}{G}{U}.
pub fn winterthorn_blessing() -> CardDefinition {
    let mine = Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) };
    let theirs = || Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou.negate()) };
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), g(), u()]))],
        ..spell(
            "Winterthorn Blessing",
            cost(&[g(), u()]),
            CardType::Sorcery,
            // Both slots are "up to one" (Boom Box's `OptionalTargets { min: 0 }`).
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter { what: mine, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::Tap { what: theirs() },
                    Effect::SkipNextUntap { what: theirs() },
                ])),
            },
        )
    }
}

/// Uneasy Partings — {3}{U} instant. {1} less if it targets an attacking
/// nontoken creature. Target creature's owner puts it on top or bottom of
/// their library (Run Behind's `OwnerChoice`).
pub fn uneasy_partings() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_target: Some((
            R::Creature.and(R::IsAttacking).and(R::Not(Box::new(R::IsToken))),
            1,
        )),
        ..spell(
            "Uneasy Partings",
            cost(&[generic(3), u()]),
            CardType::Instant,
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::OwnerChoice,
                },
            },
        )
    }
}

/// Answered Prayers — {1}{W}{W} enchantment. A creature of yours entering: gain
/// 1 life; if this isn't a creature, it becomes a 3/3 Angel with flying in
/// addition to its other types until end of turn.
pub fn answered_prayers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::This, filter: R::Not(Box::new(R::Creature)) },
                    then: Box::new(Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        creature_types: vec![CreatureType::Angel],
                        keywords: vec![Keyword::Flying],
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..spell("Answered Prayers", cost(&[generic(1), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Reaping Willow — {1}{W/B}{W/B}{W/B} 3/6 lifelink Treefolk Cleric; enters
/// with two -1/-1 counters. {1}{W/B}, remove two counters from it: reanimate
/// target creature card with MV 3 or less from your graveyard. Sorcery speed.
pub fn reaping_willow() -> CardDefinition {
    let wb = ManaSymbol::Hybrid(Color::White, Color::Black);
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), wb]),
            // "Remove two counters from this creature" — any kinds, the source
            // only (Glen Elendra Guardian's shape).
            remove_counter_among_filter: Some((None, 2, R::OtherThanSource.negate())),
            sorcery_speed: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Reaping Willow",
            cost(&[generic(1), wb, wb, wb]),
            vec![CreatureType::Treefolk, CreatureType::Cleric],
            3,
            6,
        )
    }
}

/// Falcon's Wing Harness — {1}{U} Equipment. ETB: attach to target creature
/// you control. Equipped creature gets +1/+1, flying and ward {1}. Equip {2}{U}.
pub fn falcons_wing_harness() -> CardDefinition {
    CardDefinition {
        name: "Falcon's Wing Harness",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2), u()]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::generic(1))],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        ..Default::default()
    }
}

/// Battlefront Krushok — {4}{G} 3/4 Beast. It, and each creature you control
/// with a +1/+1 counter, can't be blocked by more than one creature.
pub fn battlefront_krushok() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Each creature you control with a +1/+1 counter on it can't be blocked by more than one creature.",
            effect: crate::effect::StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::WithCounterAtLeast(CounterType::PlusOnePlusOne, 1)),
                ),
                keyword: Keyword::CantBeBlockedByMoreThanOne,
            },
        }],
        ..creature("Battlefront Krushok", cost(&[generic(4), g()]), vec![CreatureType::Beast], 3, 4)
    }
}

/// Blessings of Nature — {4}{G} sorcery. Distribute four +1/+1 counters among
/// any number of target creatures. Miracle {G}.
pub fn blessings_of_nature() -> CardDefinition {
    CardDefinition {
        miracle: Some(cost(&[g()])),
        ..spell(
            "Blessings of Nature",
            cost(&[generic(4), g()]),
            CardType::Sorcery,
            Effect::DistributeCounters {
                total: Value::Const(4),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature,
                max_targets: 4,
            },
        )
    }
}

/// Baleful Beholder — {4}{B}{B} 6/5 Beholder. ETB, choose one — each opponent
/// sacrifices an enchantment; or creatures you control gain menace until EOT.
pub fn baleful_beholder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Enchantment,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature("Baleful Beholder", cost(&[generic(4), b(), b()]), vec![CreatureType::Beholder], 6, 5)
    }
}
