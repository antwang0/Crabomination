//! Commander: the cards the **The Hosts of Mordor** precon (LTC, Sauron, Lord
//! of the Rings) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_sauron.rs`.
//!
//! Residuals (each also on its card):
//! - **Moria Scavenger** — its one ability is two: "discard a creature card:
//!   draw, amass Orcs 1" and "discard a card: draw". A creature discarded
//!   through the second one amasses nothing.
//! - **Shelob, Dread Weaver** — "put a creature card exiled with Shelob into
//!   its owner's graveyard" is paid on resolution, gated on one being there;
//!   the X ability's card is the engine's pick rather than a target.
//! - **Summons of Saruman** — flashback pays its X in mana rather than by
//!   exiling X cards from your graveyard.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, on_attack, on_cast, on_dies, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, RevealMissDest, ZoneDest};
use crate::mana::{b, cost, generic, r, u, x, Color, ManaCost};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn yours() -> R {
    R::Creature.and(R::ControlledByYou)
}

/// Amass Orcs `n` (CR 701.47).
fn amass(count: Value) -> Effect {
    Effect::Amass { who: PlayerRef::You, count, extra_type: Some(CreatureType::Orc) }
}

fn tempt() -> Effect {
    Effect::RingTempts { who: PlayerRef::You }
}

/// "When [you cast] an instant or sorcery spell" — the trigger filter.
fn cast_instant_or_sorcery(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: instant_or_sorcery() }),
        effect,
    }
}

fn on_combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

fn on_cycle(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource), effect }
}

/// Threaten the resolved creature: gain control until end of turn, untap it,
/// it gains haste until end of turn.
fn threaten(what: Selector) -> Effect {
    Effect::Seq(vec![
        Effect::GainControl { what: what.clone(), to: None, duration: Duration::EndOfTurn },
        Effect::Untap { what: what.clone(), up_to: None },
        Effect::GrantKeyword { what, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
    ])
}

fn cast_free(what: Selector, source_zone: Zone) -> Effect {
    Effect::CastWithoutPayingImmediate {
        reduce_generic: 0,
        pay_own_cost: false,
        what,
        source_zone,
        exile_after: false,
        copy: false,
    }
}

fn wraith_token() -> TokenDefinition {
    TokenDefinition {
        name: "Wraith".into(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wraith], ..Default::default() },
        ..Default::default()
    }
}

fn wraith() -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(wraith_token()) }
}

fn treasures(n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: Arc::new(crabomination_base::tokens::treasure_token()),
    }
}

/// Sauron, Lord of the Rings — on cast: amass Orcs 5, mill five, return a
/// creature card from your graveyard to the battlefield; trample; the Ring
/// tempts you whenever an opponent's commander dies.
pub fn sauron_lord_of_the_rings() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            on_cast(Effect::Seq(vec![
                amass(Value::Const(5)),
                Effect::Mill { who: Selector::You, amount: Value::Const(5) },
                Effect::PutGraveyardCardOntoBattlefield { filter: R::Creature.and(R::InYourGraveyard) },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander }),
                effect: tempt(),
            },
        ],
        ..legend(
            "Sauron, Lord of the Rings",
            cost(&[generic(5), u(), b(), r()]),
            vec![CreatureType::Avatar, CreatureType::Horror],
            9,
            9,
        )
    }
}

/// Cavern-Hoard Dragon — costs {X} less (X = the most artifacts one opponent
/// controls); flying, trample, haste; a Treasure per artifact the damaged
/// player controls.
pub fn cavern_hoard_dragon() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_per: Some((Value::MostControlledByAnOpponent(R::Artifact), 1)),
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByTriggerPlayer)))),
            definition: Arc::new(crabomination_base::tokens::treasure_token()),
        })],
        ..creature("Cavern-Hoard Dragon", cost(&[generic(7), r(), r()]), vec![CreatureType::Dragon], 6, 6)
    }
}

/// Corsairs of Umbar — {2}{U}: target Goblin, Orc or Pirate can't be blocked
/// this turn; combat damage to a player amasses Orcs 3.
pub fn corsairs_of_umbar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(
                    R::HasCreatureType(CreatureType::Goblin)
                        .or(R::HasCreatureType(CreatureType::Orc))
                        .or(R::HasCreatureType(CreatureType::Pirate)),
                )),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![on_combat_damage_to_player(amass(Value::Const(3)))],
        ..creature(
            "Corsairs of Umbar",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            3,
            3,
        )
    }
}

/// Fiery Inscription — the Ring tempts you on entry; each instant or sorcery
/// you cast deals 2 damage to each opponent.
pub fn fiery_inscription() -> CardDefinition {
    CardDefinition {
        name: "Fiery Inscription",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(tempt()),
            cast_instant_or_sorcery(Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            }),
        ],
        ..Default::default()
    }
}

/// Grishnákh, Brash Instigator — on entry, amass Orcs 2; when you do, steal a
/// nonlegendary creature an opponent controls with power at most the Army's
/// until end of turn, untapped and hasty.
pub fn grishnakh_brash_instigator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            amass(Value::Const(2)),
            Effect::WithX {
                // Your Army (you control at most one once amass has run).
                x: Value::PowerOf(Box::new(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Army).and(R::ControlledByYou),
                ))),
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::Creature
                            .and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary))))
                            .and(R::ControlledByOpponent)
                            .and(R::PowerAtMostXFromCost),
                        effect: Box::new(threaten(Selector::Target(0))),
                    }),
                }),
            },
        ]))],
        ..legend(
            "Grishnákh, Brash Instigator",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Gríma, Saruman's Footman — can't be blocked; on combat damage to a player,
/// that player exiles from the top until an instant or sorcery, you may cast
/// it free, and the rest go to the bottom in a random order.
pub fn grima_sarumans_footman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::Seq(vec![
            Effect::RevealUntilFind {
                who: PlayerRef::TriggerEventPlayer,
                find: instant_or_sorcery(),
                to: ZoneDest::Exile,
                cap: Value::Const(60),
                life_per_revealed: 0,
                miss_dest: RevealMissDest::BottomRandom,
            },
            cast_free(Selector::ExiledThisResolution { filter: instant_or_sorcery() }, Zone::Exile),
            // Not cast: it joins the others on the bottom.
            Effect::Move {
                what: Selector::ExiledThisResolution { filter: instant_or_sorcery() },
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
            },
        ]))],
        ..legend(
            "Gríma, Saruman's Footman",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            4,
        )
    }
}

/// In the Darkness Bind Them — Saga: I-III a 3/3 menace Wraith and the Ring
/// tempts you; IV steal up to one creature per opponent until end of turn,
/// then the Ring tempts you.
pub fn in_the_darkness_bind_them() -> CardDefinition {
    let wraith_and_tempt = || Effect::Seq(vec![wraith(), tempt()]);
    CardDefinition {
        name: "In the Darkness Bind Them",
        cost: cost(&[generic(2), u(), b(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, wraith_and_tempt()),
            (2, wraith_and_tempt()),
            (3, wraith_and_tempt()),
            (
                4,
                Effect::Seq(vec![
                    Effect::ForEachOpponentTarget {
                        body: Box::new(Effect::ApplyToTargets {
                            max_targets: 8,
                            min_targets: 0,
                            filter: R::Creature.and(R::ControlledByOpponent),
                            effect: Box::new(threaten(Selector::Target(0))),
                        }),
                    },
                    tempt(),
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Knollspine Dragon — flying; on entry you may discard your hand and draw
/// cards equal to the damage dealt to target opponent this turn.
pub fn knollspine_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::MayDo {
                description: "Discard your hand and draw cards equal to the damage dealt to that opponent this turn?"
                    .into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::HandSizeOf(PlayerRef::You),
                        random: false,
                    },
                    Effect::Draw { who: Selector::You, amount: Value::DamageTakenThisTurn(PlayerRef::Target(0)) },
                ])),
            }),
        })],
        ..creature("Knollspine Dragon", cost(&[generic(5), r(), r()]), vec![CreatureType::Dragon], 7, 5)
    }
}

/// Lidless Gaze — exile the top card of each player's library; until the end
/// of your next turn you may play them, spending mana as any type; flashback.
pub fn lidless_gaze() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(2), b(), r()]))],
        ..spell(
            "Lidless Gaze",
            cost(&[generic(2), b(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                    link_to_source: false,
                    face_down: false,
                },
                Effect::GrantMayPlay {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: true,
                },
            ]),
        )
    }
}

/// Lord of the Nazgûl — flying; Wraiths you control have protection from
/// Ring-bearers; each instant or sorcery you cast makes a 3/3 menace Wraith,
/// then at nine Wraiths they have base 9/9 until end of turn.
pub fn lord_of_the_nazgul() -> CardDefinition {
    let wraiths = || R::Creature.and(R::HasCreatureType(CreatureType::Wraith)).and(R::ControlledByYou);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Wraiths you control have protection from Ring-bearers.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(wraiths()),
                keyword: Keyword::ProtectionFromMatching(Box::new(R::IsRingBearer)),
            },
        }],
        triggered_abilities: vec![cast_instant_or_sorcery(Effect::Seq(vec![
            wraith(),
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(wraiths()),
                    n: Value::Const(9),
                },
                then: Box::new(Effect::SetBasePT {
                    what: Selector::EachPermanent(wraiths()),
                    power: Value::Const(9),
                    toughness: Value::Const(9),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..legend(
            "Lord of the Nazgûl",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Wraith, CreatureType::Noble],
            4,
            3,
        )
    }
}

/// Monstrosity of the Lake — on entry you may pay {5} to tap every opponent's
/// creature and put a stun counter on each; islandcycling {2}.
pub fn monstrosity_of_the_lake() -> CardDefinition {
    let theirs = || Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent));
    CardDefinition {
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Island)],
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {5} to tap and stun every creature your opponents control?".into(),
            mana_cost: cost(&[generic(5)]),
            body: Box::new(Effect::Seq(vec![
                Effect::Tap { what: theirs() },
                Effect::AddCounter { what: theirs(), kind: CounterType::Stun, amount: Value::ONE },
            ])),
            else_: None,
        })],
        ..legend("Monstrosity of the Lake", cost(&[generic(4), u()]), vec![CreatureType::Kraken], 4, 6)
    }
}

/// Moria Scavenger — deathtouch, haste; {T}, discard a card: draw a card, and
/// amass Orcs 1 if the discarded card was a creature card.
///
/// Residual: the one ability is two — a creature discarded through the plain
/// one amasses nothing.
pub fn moria_scavenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Haste],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                discard_cost: Some((R::Creature, 1)),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    amass(Value::ONE),
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..creature(
            "Moria Scavenger",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Orc, CreatureType::Rogue],
            1,
            4,
        )
    }
}

/// Orcish Siegemaster — trample; Orcs and Goblins you control have trample;
/// attacking, it gets +X/+0 (X = the greatest power among your creatures).
pub fn orcish_siegemaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Other Orcs and Goblins you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours().and(
                    R::HasCreatureType(CreatureType::Orc).or(R::HasCreatureType(CreatureType::Goblin)),
                )),
                keyword: Keyword::Trample,
            },
        }],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::GreatestPowerControlled { who: PlayerRef::You },
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Orcish Siegemaster",
            cost(&[generic(2), r()]),
            vec![CreatureType::Orc, CreatureType::Soldier],
            0,
            5,
        )
    }
}

/// Rampaging War Mammoth — trample; cycling {X}{2}{R}; cycling it destroys up
/// to X target artifacts.
pub fn rampaging_war_mammoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Cycling(cost(&[x(), generic(2), r()]))],
        triggered_abilities: vec![on_cycle(Effect::CapTargetsAt {
            amount: Value::TriggerEventAmount,
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Artifact,
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            }),
        })],
        ..creature("Rampaging War Mammoth", cost(&[generic(5), r(), r()]), vec![CreatureType::Elephant], 9, 7)
    }
}

/// Relic of Sauron — {T}: two mana in any combination of {U}, {B} and {R};
/// {3}, {T}: draw two, then discard one.
pub fn relic_of_sauron() -> CardDefinition {
    CardDefinition {
        name: "Relic of Sauron",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Blue, Color::Black, Color::Red], Value::Const(2)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Revenge of Ravens — whenever a creature attacks you or a planeswalker you
/// control, its controller loses 1 life and you gain 1 life.
pub fn revenge_of_ravens() -> CardDefinition {
    let toll = |scope| TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, scope),
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]),
    };
    CardDefinition {
        name: "Revenge of Ravens",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            toll(EventScope::ControllerAttackedByOpponent),
            toll(EventScope::ControllerPlaneswalkerAttackedByOpponent),
        ],
        ..Default::default()
    }
}

/// Saruman, the White Hand — each noncreature spell you cast amasses Orcs X
/// (its mana value); Goblins and Orcs you control have ward {2}.
pub fn saruman_the_white_hand() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Goblins and Orcs you control have ward {2}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours().and(
                    R::HasCreatureType(CreatureType::Goblin).or(R::HasCreatureType(CreatureType::Orc)),
                )),
                keyword: Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::Not(
                Box::new(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            )),
            effect: amass(Value::ManaValueOf(Box::new(Selector::TriggerSource))),
        }],
        ..legend(
            "Saruman, the White Hand",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            2,
            5,
        )
    }
}

/// Shelob, Dread Weaver — exiles each nontoken creature an opponent controls
/// that dies; {2}{B}, put one of those cards into its owner's graveyard: two
/// +1/+1 counters and a card; {X}{1}{B}: put one with mana value X onto the
/// battlefield tapped under your control.
///
/// Residual: the graveyard half of the first ability is paid on resolution
/// (the ability needs such a card to activate), and the X ability's card is
/// the engine's pick, not a target.
pub fn shelob_dread_weaver() -> CardDefinition {
    let exiled_creatures = |filter: R| Selector::MatchingAmong {
        inner: Box::new(Selector::CardExiledWithSource),
        filter: R::Creature.and(filter),
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken }),
            effect: Effect::ExileWithSource { what: Selector::TriggerSource },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                condition: Some(Predicate::SelectorExists(exiled_creatures(R::Any))),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Take { inner: Box::new(exiled_creatures(R::Any)), count: Box::new(Value::ONE) },
                        to: ZoneDest::Graveyard,
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[x(), generic(1), b()]),
                effect: Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(exiled_creatures(R::ManaValueExactlyXFromCost)),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Shelob, Dread Weaver",
            cost(&[generic(3), b()]),
            vec![CreatureType::Spider, CreatureType::Demon],
            3,
            3,
        )
    }
}

/// Subjugate the Hobbits — gain control of each noncommander creature with
/// mana value 3 or less.
pub fn subjugate_the_hobbits() -> CardDefinition {
    spell(
        "Subjugate the Hobbits",
        cost(&[generic(5), u(), u()]),
        CardType::Sorcery,
        Effect::GainControl {
            what: Selector::EachPermanent(
                R::Creature.and(R::Not(Box::new(R::IsCommander))).and(R::ManaValueAtMost(3)),
            ),
            to: None,
            duration: Duration::Permanent,
        },
    )
}

/// Summons of Saruman — amass Orcs X, mill X, then you may cast an instant or
/// sorcery with mana value X or less from among the milled cards free.
///
/// Residual: flashback pays X in mana instead of exiling X graveyard cards.
pub fn summons_of_saruman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[x(), generic(3), u(), r()]))],
        ..spell(
            "Summons of Saruman",
            cost(&[x(), u(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                amass(Value::XFromCost),
                Effect::Mill { who: Selector::You, amount: Value::XFromCost },
                cast_free(
                    Selector::MatchingAmong {
                        inner: Box::new(Selector::LastMoved),
                        filter: instant_or_sorcery().and(R::ManaValueAtMostXFromCost),
                    },
                    Zone::Graveyard,
                ),
            ]),
        )
    }
}

/// The Balrog of Moria — trample, haste; when it dies you may exile it, and
/// when you do, exile up to one creature per opponent; cycling {3}{R}, which
/// makes two Treasures.
pub fn the_balrog_of_moria() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Cycling(cost(&[generic(3), r()]))],
        triggered_abilities: vec![
            on_dies(Effect::MayExileSelfThen {
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::ForEachOpponentTarget {
                        body: Box::new(Effect::ApplyToTargets {
                            max_targets: 8,
                            min_targets: 0,
                            filter: R::Creature.and(R::ControlledByOpponent),
                            effect: Box::new(Effect::Exile { what: Selector::Target(0) }),
                        }),
                    }),
                }),
            }),
            on_cycle(treasures(2)),
        ],
        ..legend(
            "The Balrog of Moria",
            cost(&[generic(4), b(), b(), r()]),
            vec![CreatureType::Avatar, CreatureType::Demon],
            8,
            8,
        )
    }
}

/// The Mouth of Sauron — on entry, target player mills three, then amass Orcs
/// X (the instants and sorceries in that player's graveyard).
pub fn the_mouth_of_sauron() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(Effect::Seq(vec![
                Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(3) },
                amass(Value::CardsInGraveyardMatching { who: PlayerRef::Target(0), filter: instant_or_sorcery() }),
            ])),
        })],
        ..legend(
            "The Mouth of Sauron",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            3,
            4,
        )
    }
}

/// Too Greedily, Too Deep — put a creature card from a graveyard onto the
/// battlefield under your control; it deals damage equal to its power to
/// each other creature.
pub fn too_greedily_too_deep() -> CardDefinition {
    spell(
        "Too Greedily, Too Deep",
        cost(&[generic(5), b(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.from_any_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::DealDamageEqualToPowerToEach {
                source: Selector::LastMoved,
                targets: Selector::EachPermanent(R::Creature),
                each_opponent: false,
            },
        ]),
    )
}

/// Treason of Isengard — put up to one instant or sorcery card from your
/// graveyard on top of your library; amass Orcs 2.
pub fn treason_of_isengard() -> CardDefinition {
    spell(
        "Treason of Isengard",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Move {
                    what: target_filtered(instant_or_sorcery().and(R::InYourGraveyard)),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                }),
            },
            amass(Value::Const(2)),
        ]),
    )
}

/// Wake the Dragon — a 6/6 flying, menace Dragon that steals an artifact from
/// each player it deals combat damage to; flashback {6}{B}{R}.
pub fn wake_the_dragon() -> CardDefinition {
    let dragon = TokenDefinition {
        name: "Dragon".into(),
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Menace],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        triggered_abilities: vec![on_combat_damage_to_player(Effect::GainControl {
            what: target_filtered(R::Artifact.and(R::ControlledByTriggerPlayer)),
            to: None,
            duration: Duration::Permanent,
        })],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(6), b(), r()]))],
        ..spell(
            "Wake the Dragon",
            cost(&[generic(4), b(), r()]),
            CardType::Sorcery,
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(dragon) },
        )
    }
}
