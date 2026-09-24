//! Commander: the cards the **Mishra's Burnished Banner** precon (BRC,
//! Mishra, Eminent One) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_mishra.rs`.
//!
//! Residuals (each also on its card):
//! - **Mishra, Eminent One** — the Warform keeps the copied artifact's name;
//!   it isn't legendary, so the legend rule leaves it alone as the printed
//!   rename would.
//! - **Ashnod the Uncaring** — the copy is found through the ability's source,
//!   so an ability whose source was itself sacrificed can't be copied.
//! - **Blast-Furnace Hellkite** — "creatures attacking your opponents" also
//!   counts creatures attacking an opponent's planeswalker.
//! - **Smelting Vat** — each card is capped at the sacrificed artifact's mana
//!   value, not the pair's total.
//! - **Lithoform Engine** — the ability copy is Strionic Resonator's (the
//!   target is the source permanent, the copy keeps its targets).
//! - **Workshop Elders** — its flying grant matches printed card types, so an
//!   artifact it (or anything) animates doesn't fly (CR 613.8 dependency).
//! - **Glint Raker** — the reveal isn't optional.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EntersAsCopy, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Predicate, ZoneDest, ZoneRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u, x, Color, ManaCost};
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

fn mint(who: PlayerRef, count: Value, token: TokenDefinition) -> Effect {
    Effect::CreateToken { who, count, definition: Arc::new(token) }
}

fn your_step(step: TurnStep) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl)
}

fn sac_artifact() -> Option<(R, u32)> {
    Some((R::Artifact, 1))
}

fn minus(n: Value) -> Value {
    Value::Diff(Box::new(Value::Const(0)), Box::new(n))
}

fn construct_token() -> TokenDefinition {
    TokenDefinition {
        name: "Construct".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        ..Default::default()
    }
}

/// Mishra, Eminent One — each combat on your turn, a hasty 4/4 Construct copy
/// of your noncreature artifact, sacrificed at the next end step. Residual:
/// the Warform keeps the artifact's name (non-legendary, so the legend rule
/// stays out of it as the rename would keep it out).
pub fn mishra_eminent_one() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::BeginCombat),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(
                        R::Artifact.and(R::Not(Box::new(R::Creature))).and(R::ControlledByYou),
                    ),
                    extra_creature_types: vec![CreatureType::Construct],
                    extra_card_types: vec![CardType::Creature],
                    override_pt: Some((4, 4)),
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: true,
                    legendary: false,
                    extra_keywords: vec![],
                },
                Effect::GrantKeyword {
                    what: Selector::LastCreatedToken,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedToken },
            ]),
        }],
        ..creature(
            "Mishra, Eminent One",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            5,
            4,
        )
    }
}

/// Ashnod the Uncaring — deathtouch; activating a non-mana ability of an
/// artifact or creature with a sacrifice in its cost may copy it. Residual: an
/// ability whose source was the thing sacrificed can't be found to copy.
pub fn ashnod_the_uncaring() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivatedWithSacrifice, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.or(R::Creature),
                }),
            effect: Effect::MayDo {
                description: "Copy that ability?".into(),
                body: Box::new(Effect::CopyAbility { what: Selector::TriggerSource, times: Value::ONE }),
            },
        }],
        ..creature(
            "Ashnod the Uncaring",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            4,
        )
    }
}

/// Blast-Furnace Hellkite — artifact offering; flying, double strike;
/// creatures attacking your opponents have double strike.
pub fn blast_furnace_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(7), r(), r()]),
            offering: Some(R::Artifact),
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Creatures attacking your opponents have double strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsAttackingAnOpponent)),
                keyword: Keyword::DoubleStrike,
            },
        }],
        ..creature("Blast-Furnace Hellkite", cost(&[generic(7), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Fain, the Broker — four tap abilities: sacrifice a creature for two
/// counters, a counter for a Treasure, an artifact for a 2/1 Inkling; {3}{B}
/// untaps it.
pub fn fain_the_broker() -> CardDefinition {
    let inkling = TokenDefinition {
        name: "Inkling".into(),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Inkling], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_among_filter: Some((None, 1, R::Creature)),
                effect: crate::effect::shortcut::mint_treasures(1),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: sac_artifact(),
                effect: mint(PlayerRef::You, Value::ONE, inkling),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b()]),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
        ],
        ..creature(
            "Fain, the Broker",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Farid, Enterprising Salvager — a Scrap for each nontoken artifact of yours
/// that dies; {1}{R}, sacrifice an artifact: a counter and menace, a goad, or
/// a loot.
pub fn farid_enterprising_salvager() -> CardDefinition {
    let scrap = TokenDefinition { name: "Scrap".into(), card_types: vec![CardType::Artifact], ..Default::default() };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::NotToken) },
            ),
            effect: mint(PlayerRef::You, Value::ONE, scrap),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_other_filter: sac_artifact(),
            effect: Effect::ChooseMode(vec![
                Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Menace, duration: Duration::EndOfTurn },
                ]),
                Effect::Goad { what: target_filtered(R::Creature) },
                Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Farid, Enterprising Salvager",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Geth, Lord of the Vault — intimidate; {X}{B}: an opponent's artifact or
/// creature card with mana value X comes to you tapped, and its owner mills X.
pub fn geth_lord_of_the_vault() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Intimidate],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), b()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Artifact
                            .or(R::Creature)
                            .and(R::InOpponentGraveyard)
                            .and(R::ManaValueExactlyXFromCost),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::Mill {
                    who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    amount: Value::XFromCost,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Geth, Lord of the Vault",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie],
            5,
            5,
        )
    }
}

/// Glint Raker — flying; +X/+0, X the greatest mana value among your
/// artifacts; connecting, look at that many and keep an artifact.
pub fn glint_raker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +X/+0, where X is the greatest mana value among artifacts you control.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::HighestManaValueAmong(Box::new(Selector::EachPermanent(
                    R::Artifact.and(R::ControlledByYou),
                ))),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                rest_to_graveyard: true,
                pick_filter: Some(R::Artifact),
                ..Default::default()
            })),
        }],
        ..creature("Glint Raker", cost(&[generic(3), u()]), vec![CreatureType::Drake], 1, 3)
    }
}

/// Herald of Anguish — improvise, flying; each opponent discards at your end
/// step; {1}{B}, sacrifice an artifact: -2/-2.
pub fn herald_of_anguish() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Improvise, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: sac_artifact(),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Herald of Anguish", cost(&[generic(5), b(), b()]), vec![CreatureType::Demon], 5, 5)
    }
}

/// Lithoform Engine — copy an ability, an instant or sorcery, or a permanent
/// spell (a token) you control. Residual: the ability copy is Strionic
/// Resonator's.
pub fn lithoform_engine() -> CardDefinition {
    let spell = |filter: R| target_filtered(R::IsSpellOnStack.and(R::ControlledByYou).and(filter));
    CardDefinition {
        name: "Lithoform Engine",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::CopyAbility {
                    what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByYou)),
                    times: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::CopySpellMayChooseTargets {
                    what: spell(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                    count: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::CopySpell { what: spell(R::Permanent), count: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Machine God's Effigy — may enter as a copy of any creature, except an
/// artifact that isn't a creature and taps for {U}.
pub fn machine_gods_effigy() -> CardDefinition {
    CardDefinition {
        name: "Machine God's Effigy",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_card_types: vec![CardType::Artifact],
            extra_activated: vec![crate::sets::tap_add(Color::Blue)],
            not_a_creature: true,
            ..Default::default()
        }),
        activated_abilities: vec![crate::sets::tap_add(Color::Blue)],
        ..Default::default()
    }
}

/// Oni-Cult Anvil — once a turn on your turn, a Construct when your artifacts
/// leave; {T}, sacrifice an artifact: 1 damage to each opponent, gain 1.
pub fn oni_cult_anvil() -> CardDefinition {
    CardDefinition {
        name: "Oni-Cult Anvil",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
                    Predicate::SamePlayer(PlayerRef::ActivePlayer, PlayerRef::You),
                ]))
                .once_per_turn(),
            effect: mint(PlayerRef::You, Value::ONE, construct_token()),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: sac_artifact(),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Scavenged Brawler — flying, vigilance, trample, lifelink; from the
/// graveyard, {5} and exile it: four +1/+1 counters and those four keyword
/// counters on a creature (sorcery speed).
pub fn scavenged_brawler() -> CardDefinition {
    let kw = |k: Keyword| Effect::AddKeywordCounter { what: Selector::Target(0), keyword: k, amount: Value::ONE };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(4),
                },
                kw(Keyword::Flying),
                kw(Keyword::Vigilance),
                kw(Keyword::Trample),
                kw(Keyword::Lifelink),
            ]),
            ..Default::default()
        }],
        ..creature("Scavenged Brawler", cost(&[generic(6)]), vec![CreatureType::Construct], 4, 4)
    }
}

/// Smelting Vat — {1}, {T}, sacrifice another artifact: up to two noncreature
/// artifact cards from the top eight onto the battlefield. Residual: each is
/// capped at the sacrificed artifact's mana value, not their total.
pub fn smelting_vat() -> CardDefinition {
    CardDefinition {
        name: "Smelting Vat",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact.and(R::OtherThanSource), 1)),
            effect: Effect::WithX {
                x: Value::SacrificedManaValue,
                body: Box::new(Effect::LookTopPutMatchingOntoBattlefield {
                    count: Value::Const(8),
                    filter: R::Artifact.and(R::Not(Box::new(R::Creature))).and(R::ManaValueAtMostXFromCost),
                    then: None,
                    max: Some(2),
                    tapped: false,
                    exile_rest: false,
                    rest_to_graveyard: false,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Terisiare's Devastation — lose X, X tapped Powerstones, then every creature
/// gets -1/-1 per artifact you control.
pub fn terisiares_devastation() -> CardDefinition {
    let mut stone = crate::game::effects::powerstone_token();
    stone.tapped = true;
    let per_artifact = || Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))));
    CardDefinition {
        name: "Terisiare's Devastation",
        cost: cost(&[x(), generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::XFromCost },
            mint(PlayerRef::You, Value::XFromCost, stone),
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: minus(per_artifact()),
                toughness: minus(per_artifact()),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Traxos, Scourge of Kroog — trample; enters tapped and doesn't untap
/// normally; each historic spell you cast untaps it.
pub fn traxos_scourge_of_kroog() -> CardDefinition {
    let historic = R::Artifact
        .or(R::HasSupertype(Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Saga));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Trample],
        static_abilities: vec![
            crate::sets::enters_tapped(),
            StaticAbility {
                description: "Traxos doesn't untap during your untap step.",
                effect: StaticEffect::PreventUntap { applies_to: Selector::This },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(historic)),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..creature("Traxos, Scourge of Kroog", cost(&[generic(4)]), vec![CreatureType::Construct], 7, 7)
    }
}

/// Wondrous Crucible — your permanents have ward {2}; at your end step mill
/// two, exile a random nonland card from your graveyard, and you may cast a
/// copy of it free.
pub fn wondrous_crucible() -> CardDefinition {
    CardDefinition {
        name: "Wondrous Crucible",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Permanents you control have ward {2}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::ControlledByYou),
                keyword: Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                Effect::Move {
                    what: Selector::TakeRandom {
                        inner: Box::new(Selector::EachMatching {
                            zone: ZoneRef::Graveyard(PlayerRef::You),
                            filter: R::Nonland,
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Exile,
                },
                Effect::CastWithoutPayingImmediate {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    source_zone: crate::card::Zone::Exile,
                    exile_after: false,
                    copy: true,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Workshop Elders — your artifact creatures fly; each combat on your turn a
/// noncreature artifact of yours may become a 0/0 artifact creature with four
/// +1/+1 counters. Residual: the flying grant reads printed types, so an
/// animated artifact doesn't get it.
pub fn workshop_elders() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
                keyword: Keyword::Flying,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::BeginCombat),
            effect: Effect::MayDo {
                description: "Animate a noncreature artifact with four +1/+1 counters?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: target_filtered(
                            R::Artifact.and(R::Not(Box::new(R::Creature))).and(R::ControlledByYou),
                        ),
                        power: Value::Const(0),
                        toughness: Value::Const(0),
                        creature_types: vec![],
                        keywords: vec![],
                        duration: Duration::Permanent,
                    },
                    Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(4),
                    },
                ])),
            },
        }],
        ..creature(
            "Workshop Elders",
            cost(&[generic(6), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            4,
            4,
        )
    }
}
