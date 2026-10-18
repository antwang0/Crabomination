//! Commander: 2026-10-17 (`019Y6J4R`) closers, third batch. Tests in
//! `tests/recent_b/cmdr_019y_c.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    Predicate, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, WardCost, Zone,
};
use crate::effect::shortcut::{boast, drain_target_opponent, etb, on_attack, on_dies, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, w, x};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn token(def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(def) }
}

fn your_step(s: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(s), EventScope::YourControl), effect }
}

/// Clattering Skeletons — {3}{B} 4/3 Skeleton. Dies: venture into the dungeon.
pub fn clattering_skeletons() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Venture)],
        ..creature("Clattering Skeletons", cost(&[generic(3), b()]), vec![CreatureType::Skeleton], 4, 3)
    }
}

/// Starnheim Unleashed — {2}{W}{W} sorcery, foretell {X}{X}{W}. A 4/4 white
/// flying, vigilant Angel Warrior; X of them if it was foretold (CR 702.143).
pub fn starnheim_unleashed() -> CardDefinition {
    let angel = TokenDefinition {
        name: "Angel Warrior".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        foretell_cost: Some(cost(&[x(), x(), w()])),
        ..spell(
            "Starnheim Unleashed",
            cost(&[generic(2), w(), w()]),
            CardType::Sorcery,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::IfPred {
                    pred: Box::new(Predicate::CastForetold),
                    then: Box::new(Value::XFromCost),
                    else_: Box::new(Value::ONE),
                },
                definition: Arc::new(angel),
            },
        )
    }
}

/// Barrowin of Clan Undurr — {2}{W}{B} legendary 3/3 Dwarf Cleric. ETB:
/// venture. Attacking, once you've completed a dungeon: return up to one
/// creature card with mana value 3 or less from your graveyard to the
/// battlefield.
pub fn barrowin_of_clan_undurr() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(Effect::Venture),
            on_attack(Effect::If {
                cond: Predicate::ValueAtLeast(Value::DungeonsCompleted, Value::ONE),
                then: Box::new(Effect::MoveChosen {
                    from: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Creature.and(R::ManaValueAtMost(3)),
                    },
                    filter: None,
                    count: Value::ONE,
                    up_to: true,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..creature(
            "Barrowin of Clan Undurr",
            cost(&[generic(2), w(), b()]),
            vec![CreatureType::Dwarf, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Seraphic Greatsword — {1}{W} Equipment, equip {4}. +2/+2; whenever the
/// equipped creature attacks the player with the most life (or tied), a 4/4
/// white flying Angel enters tapped and attacking that player.
pub fn seraphic_greatsword() -> CardDefinition {
    let angel = TokenDefinition {
        name: "Angel".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Seraphic Greatsword",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                // CR 702.105a's dethrone shape: the player, not a planeswalker.
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                    Predicate::All(vec![
                        Predicate::EntityMatches { what: Selector::This, filter: R::IsAttackingOpponentPlayer },
                        Predicate::PlayerHasMostLife { who: PlayerRef::DefendingPlayer },
                    ]),
                ),
                effect: Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(angel),
                    cleanup: Default::default(),
                    defender: Some(PlayerRef::DefendingPlayer),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Duskwielder — {B} 1/2 Elf Berserker. Boast — {1}: target opponent loses 1
/// life and you gain 1 life.
pub fn duskwielder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![boast(cost(&[generic(1)]), drain_target_opponent(1))],
        ..creature("Duskwielder", cost(&[b()]), vec![CreatureType::Elf, CreatureType::Berserker], 1, 2)
    }
}

/// Varis, Silverymoon Ranger — {1}{G}{G} legendary 3/3, reach, ward {1}. Your
/// first creature or planeswalker spell each turn ventures; completing a
/// dungeon makes a 2/2 green Wolf.
pub fn varis_silverymoon_ranger() -> CardDefinition {
    let wolf = TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Reach, Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.or(R::Planeswalker),
                    })
                    .once_per_turn(),
                effect: Effect::Venture,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DungeonCompleted, EventScope::YourControl),
                effect: token(wolf),
            },
        ],
        ..creature(
            "Varis, Silverymoon Ranger",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Ranger],
            3,
            3,
        )
    }
}

/// Black Widow, Double Agent — {1}{W}{B} legendary 3/2 deathtouch. A creature
/// you control attacking alone gains first strike and menace until end of turn.
pub fn black_widow_double_agent() -> CardDefinition {
    let gain = |kw| Effect::GrantKeyword { what: Selector::TriggerSource, keyword: kw, duration: Duration::EndOfTurn };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::AttackingAlone),
            effect: Effect::Seq(vec![gain(Keyword::FirstStrike), gain(Keyword::Menace)]),
        }],
        ..creature(
            "Black Widow, Double Agent",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Hero, CreatureType::Villain],
            3,
            2,
        )
    }
}

/// Timecrafting — {X}{R} instant. Remove X time counters from target permanent
/// or suspended card; or put X time counters on target permanent with a time
/// counter or suspended card.
pub fn timecrafting() -> CardDefinition {
    let suspended = || R::InExile.and(R::IsSuspended);
    spell(
        "Timecrafting",
        cost(&[x(), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::RemoveCounter {
                what: target_filtered(R::OnBattlefield.or(suspended())),
                kind: CounterType::Time,
                amount: Value::XFromCost,
            },
            Effect::AddCounter {
                what: target_filtered(R::OnBattlefield.and(R::WithCounter(CounterType::Time)).or(suspended())),
                kind: CounterType::Time,
                amount: Value::XFromCost,
            },
        ]),
    )
}

/// Archfiend's Vessel — {B} 1/1 lifelink Human Cleric. Entering from your
/// graveyard (or cast from it): exile it and make a 5/5 black flying Demon.
pub fn archfiends_vessel() -> CardDefinition {
    let demon = TokenDefinition {
        name: "Demon".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        ..Default::default()
    };
    let mut t = etb(Effect::If {
        cond: Predicate::EntityMatches { what: Selector::This, filter: R::OnBattlefield },
        then: Box::new(Effect::Seq(vec![
            Effect::Move { what: Selector::This, to: ZoneDest::Exile },
            token(demon),
        ])),
        else_: Box::new(Effect::Noop),
    });
    t.event = t.event.with_filter(Predicate::TriggerSourceEnteredFromGraveyard);
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![t],
        ..creature("Archfiend's Vessel", cost(&[b()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 1)
    }
}

/// Befriending the Moths // Imperial Moth — {3}{W} Saga. I, II: target
/// creature you control gets +1/+1 and flying until end of turn. III: exile it,
/// return it transformed: a 2/4 flying Insect enchantment creature.
pub fn befriending_the_moths() -> CardDefinition {
    let lift = || {
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
        ])
    };
    let moth = CardDefinition {
        name: "Imperial Moth",
        card_types: vec![CardType::Enchantment, CardType::Creature],
        color_indicator: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Befriending the Moths",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![(1, lift()), (2, lift()), (3, Effect::ExileSelfReturnTransformed)],
        back_face: Some(Box::new(moth)),
        ..Default::default()
    }
}

/// Pizza Face, Gastromancer — {3}{B}{G} legendary 2/4 artifact creature Food
/// Mutant. ETB: a Food. Disappear: three +1/+1 counters on up to one other
/// target artifact or creature, which becomes a 0/0 Mutant creature if it
/// isn't one. {10}, {T}, sacrifice it: gain 15 life.
pub fn pizza_face_gastromancer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food],
            creature_types: vec![CreatureType::Mutant],
            ..Default::default()
        },
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::food_token()),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Artifact.or(R::Creature).and(R::OtherThanSource),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::If {
                            cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                            then: Box::new(Effect::Noop),
                            else_: Box::new(Effect::BecomeCreature {
                                what: Selector::Target(0),
                                power: Value::ZERO,
                                toughness: Value::ZERO,
                                creature_types: vec![CreatureType::Mutant],
                                keywords: vec![],
                                duration: Duration::Permanent,
                            }),
                        },
                        Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(3),
                        },
                    ])),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(10)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(15) },
            ..Default::default()
        }],
        ..creature("Pizza Face, Gastromancer", cost(&[generic(3), b(), g()]), vec![CreatureType::Mutant], 2, 4)
    }
}

/// Krang & Shredder — {4}{U/B}{U/B} legendary 6/7. Entering or attacking, each
/// opponent exiles cards from the top until a nonland card (linked).
/// Disappear: you may cast a card exiled with them without paying its mana cost.
pub fn krang_and_shredder() -> CardDefinition {
    let dig = || Effect::ForEachOpponent {
        body: Box::new(Effect::ExileTopUntilNonlandWithSource { who: PlayerRef::Triggerer }),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(dig()),
            on_attack(dig()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
                effect: Effect::MayCastExiledWithSource { filter: R::Land.negate() },
            },
        ],
        ..creature(
            "Krang & Shredder",
            cost(&[
                generic(4),
                hybrid(Color::Blue, Color::Black),
                hybrid(Color::Blue, Color::Black),
            ]),
            vec![CreatureType::Utrom, CreatureType::Human, CreatureType::Ninja],
            6,
            7,
        )
    }
}

/// Curse of Hospitality — {2}{R} Aura Curse. Creatures attacking the enchanted
/// player have trample (granted as each attacks); a creature dealing it combat
/// damage exiles its top card, which that creature's controller may play this
/// turn, spending mana as any color.
pub fn curse_of_hospitality() -> CardDefinition {
    CardDefinition {
        name: "Curse of Hospitality",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(Predicate::SamePlayer(
                    PlayerRef::DefendingPlayer,
                    PlayerRef::EnchantedPlayer,
                )),
                effect: Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer).with_filter(
                    Predicate::SamePlayer(PlayerRef::TriggerEventPlayer, PlayerRef::EnchantedPlayer),
                ),
                effect: Effect::AsPlayer {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    body: Box::new(Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::EnchantedPlayer,
                        count: Value::ONE,
                        duration: MayPlayDuration::EndOfThisTurn,
                        pay_any_color: true,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Rankle and Torbran — {1}{B}{B}{R}{R} legendary 3/4 flying, first strike,
/// haste. Combat damage to a player or battle: choose any number — each player
/// creates a Treasure; each player sacrifices a creature; damage to players and
/// battles is +2 this turn.
pub fn rankle_and_torbran() -> CardDefinition {
    let each = |body| Effect::EachPlayerDoes { who: PlayerRef::EachPlayer, body: Box::new(body) };
    let modes = Effect::ChooseUpToN {
        max: Box::new(Value::Const(3)),
        modes: vec![
            each(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            }),
            each(Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature }),
            Effect::DamageToPlayersAndBattlesPlusThisTurn { amount: 2 },
        ],
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).or_battle(),
            effect: modes,
        }],
        ..creature(
            "Rankle and Torbran",
            cost(&[generic(1), b(), b(), r(), r()]),
            vec![CreatureType::Faerie, CreatureType::Dwarf],
            3,
            4,
        )
    }
}

/// Paradox Shaper // Omit Variables — {1}{U/B} 1/3 Octopus Wizard. Upkeep:
/// becomes prepared if it isn't. {2}: target card from your graveyard to the
/// bottom of your library. Omit Variables ({U/B} sorcery): mill three.
pub fn paradox_shaper() -> CardDefinition {
    let ub = || hybrid(Color::Blue, Color::Black);
    let mut upkeep = your_step(
        TurnStep::Upkeep,
        Effect::AddCounterCapped { what: Selector::This, kind: CounterType::Prepared, amount: Value::ONE, cap: Value::ONE },
    );
    upkeep.event = upkeep.event.with_filter(Predicate::EntityMatches {
        what: Selector::This,
        filter: R::WithCounter(CounterType::Prepared).negate(),
    });
    CardDefinition {
        triggered_abilities: vec![upkeep],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Move {
                what: target_filtered(R::InGraveyard.and(R::OwnedByYou)),
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Bottom },
            },
            ..Default::default()
        }],
        prepare_spell: Some(Arc::new(spell(
            "Omit Variables",
            cost(&[ub()]),
            CardType::Sorcery,
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
        ))),
        ..creature("Paradox Shaper", cost(&[generic(1), ub()]), vec![CreatureType::Octopus, CreatureType::Wizard], 1, 3)
    }
}

/// Konstrari Improviser // Soul Tether — {1}{R/G} 2/2 Human Artificer, enters
/// prepared. Soul Tether ({2}{R/G} sorcery): a Heartwood token (a red and green
/// artifact with "{T}: Add {R} or {G}").
pub fn konstrari_improviser() -> CardDefinition {
    let rg = || hybrid(Color::Red, Color::Green);
    let heartwood = TokenDefinition {
        name: "Heartwood".into(),
        card_types: vec![CardType::Artifact],
        colors: vec![Color::Red, Color::Green],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Red, Color::Green], Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        prepare_spell: Some(Arc::new(spell("Soul Tether", cost(&[generic(2), rg()]), CardType::Sorcery, token(heartwood)))),
        ..creature("Konstrari Improviser", cost(&[generic(1), rg()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 2)
    }
}
