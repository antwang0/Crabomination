//! Commander: the cards the **Blast from the Past** precon (WHO, The Fourth
//! Doctor + Sarah Jane Smith) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fourth_doctor.rs`.
//!
//! Residuals (each also on its card):
//! - **Ace's Baseball Bat** — "must be blocked by a Dalek if able" is not
//!   modelled.
//! - **Displaced Dinosaurs** — the historic permanent becomes a 7/7 Dinosaur
//!   as a trigger resolves, not as it enters.
//! - **Nyssa of Traken** — the creatures are tapped up to the count on
//!   resolution, not targeted.
//! - **Peri Brown** — every historic spell has convoke, not only the first
//!   each turn.
//! - **Reverse the Polarity** — "can't be blocked" reaches only the
//!   creatures on the battlefield as it resolves.
//! - **Susan Foreman** — the planeswalk replacement does nothing (no
//!   Planechase).
//! - **The Curse of Fenric** — I spares your own creatures; II makes a 6/6
//!   with no abilities but doesn't rename it Fenric or make it legendary, so
//!   III is a Mutant fighting any other creature.
//! - **The Eighth Doctor** — the historic land and the historic permanent
//!   spell are separate allowances, and the cast permanent isn't exiled if it
//!   leaves later.
//! - **The Fourth Doctor** — no Food for a land played from the top.
//! - **The Second Doctor** — an opponent who draws has its current creatures
//!   barred from attacking you, not ones that arrive later.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, ConditionalEquipBonus, CounterType, CreatureType,
    DynamicPt, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, LandType,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, investigate, on_attack, on_you_attack, target_filtered};
use crate::effect::{
    Duration, Effect, GoadLasts, LibraryPosition, ManaPayload, PlayerRef, Predicate, RevealMissDest, VoteOption,
    VoteTally, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, u, w, x};
use crabomination_base::tokens::{food_token, treasure_token};

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

/// A legendary Doctor's companion (CR 702.124m).
fn companion(mut def: CardDefinition) -> CardDefinition {
    def.keywords.push(Keyword::DoctorsCompanion);
    legend(def)
}

/// A legendary Time Lord Doctor.
fn doctor(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    legend(creature(name, mana, vec![CreatureType::TimeLord, CreatureType::Doctor], p, t))
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
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

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn plus(what: Selector, n: i32) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::Const(n) }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn make(t: TokenDefinition, n: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(t) }
}

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn legendary() -> R {
    R::HasSupertype(Supertype::Legendary)
}

/// Artifacts, legendaries and Sagas (CR 700.6).
fn historic() -> R {
    R::Artifact.or(legendary()).or(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga))
}

fn doctors() -> R {
    R::HasCreatureType(CreatureType::Doctor)
}

fn time_lord() -> R {
    R::HasCreatureType(CreatureType::TimeLord)
}

/// "Whenever you cast a historic spell, `effect`."
fn on_historic_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(historic())),
        effect,
    }
}

fn at(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

fn enters_tapped() -> StaticAbility {
    StaticAbility {
        description: "This permanent enters tapped.",
        effect: StaticEffect::EntersTapped { applies_to: Selector::This },
    }
}

// ── Commanders ──────────────────────────────────────────────────────────────

/// The Fourth Doctor — look at the top card any time; once each turn play a
/// historic land or cast a historic spell from there, and a spell cast that
/// way makes a Food.
///
/// ⚠ Residual: no Food for a land played from the top, and a historic spell
/// cast from the library by another permission makes one too.
pub fn the_fourth_doctor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "Once each turn, you may play a historic land or cast a historic spell from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTopOncePerTurn { filter: historic() },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellFromLibrary,
                Predicate::CastSpellMatches(historic()),
            ])),
            effect: make(food_token(), Value::ONE),
        }],
        ..doctor("The Fourth Doctor", cost(&[generic(2), g(), u()]), 4, 4)
    }
}

/// Sarah Jane Smith — investigates on your first historic spell each turn.
pub fn sarah_jane_smith() -> CardDefinition {
    let mut t = on_historic_cast(investigate(1));
    t.event = t.event.once_per_turn();
    companion(CardDefinition {
        triggered_abilities: vec![t],
        ..creature(
            "Sarah Jane Smith",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            1,
        )
    })
}

// ── The Doctors ─────────────────────────────────────────────────────────────

/// The First Doctor — fetches TARDIS from library or graveyard; a cascade
/// spell counters up an artifact or creature.
pub fn the_first_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::SearchZones {
                who: PlayerRef::You,
                zones: vec![Zone::Library, Zone::Graveyard],
                filter: R::HasName("TARDIS".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(trigger_is(R::HasKeyword(Keyword::Cascade))),
                effect: plus(target_filtered(R::Artifact.or(R::Creature)), 1),
            },
        ],
        ..doctor("The First Doctor", cost(&[generic(1), w(), u()]), 2, 2)
    }
}

/// The Second Doctor — no maximum hand size for anyone; at your end step each
/// player may draw, and an opponent who does can't attack you next turn.
///
/// ⚠ Residual: only that opponent's creatures on the battlefield then are
/// barred.
pub fn the_second_doctor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players have no maximum hand size.",
            effect: StaticEffect::AllPlayersNoMaximumHandSize,
        }],
        triggered_abilities: vec![at(
            TurnStep::End,
            Effect::Seq(vec![
                Effect::MayDoBy { who: PlayerRef::You, description: "Draw a card?".into(), body: Box::new(draw(Value::ONE)) },
                Effect::ForEachOpponent {
                    body: Box::new(Effect::MayDoBy {
                        who: PlayerRef::Triggerer,
                        description: "Draw a card? You can't attack The Second Doctor's controller next turn.".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::Draw { who: Selector::Player(PlayerRef::Triggerer), amount: Value::ONE },
                            Effect::GrantCantAttackYou {
                                what: Selector::ControlledBy { who: PlayerRef::Triggerer, filter: R::Creature },
                                duration: Duration::UntilYourNextUntap,
                            },
                        ])),
                    }),
                },
            ]),
        )],
        ..doctor("The Second Doctor", cost(&[generic(2), w(), u()]), 2, 4)
    }
}

/// The Fifth Doctor — at your end step, each creature of yours that neither
/// attacked nor entered this turn gets a +1/+1 counter and untaps.
pub fn the_fifth_doctor() -> CardDefinition {
    let rested =
        || Selector::EachPermanent(yours(R::Creature).and(R::AttackedThisTurn.negate()).and(R::EnteredThisTurn.negate()));
    CardDefinition {
        triggered_abilities: vec![at(
            TurnStep::End,
            Effect::Seq(vec![plus(rested(), 1), Effect::Untap { what: rested(), up_to: None }]),
        )],
        ..doctor("The Fifth Doctor", cost(&[generic(2), w(), u()]), 2, 2)
    }
}

/// The Sixth Doctor — copies your first historic spell each turn, the copy
/// not legendary (CR 707.9b).
pub fn the_sixth_doctor() -> CardDefinition {
    let mut t = on_historic_cast(Effect::CopySpellNonLegendary { what: Selector::TriggerSource });
    t.event = t.event.once_per_turn();
    CardDefinition { triggered_abilities: vec![t], ..doctor("The Sixth Doctor", cost(&[generic(4), g(), u()]), 3, 3) }
}

/// The Seventh Doctor — the defending player guesses a hand card's mana
/// value against your artifact count; a wrong guess casts it free, else you
/// investigate.
pub fn the_seventh_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::GuessManaValueAgainstValue {
            who: PlayerRef::DefendingPlayer,
            threshold: Value::PermanentCountControlledByMatching(PlayerRef::You, R::Artifact),
            otherwise: Box::new(investigate(1)),
        })],
        ..doctor("The Seventh Doctor", cost(&[generic(3), w(), u()]), 3, 6)
    }
}

/// The Eighth Doctor — mills three; plays historic permanents from the
/// graveyard once a turn.
///
/// ⚠ Residual: the land and the spell are separate allowances, and the cast
/// permanent isn't exiled if it leaves later.
pub fn the_eighth_doctor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Once during each of your turns, you may cast a historic permanent spell from your graveyard.",
                effect: StaticEffect::GraveyardCastOncePerTurn {
                    filter: historic().and(R::PermanentCard),
                    exile_after: true,
                    mv_at_most_counters: None,
                },
            },
            StaticAbility {
                description: "You may play historic lands from your graveyard.",
                effect: StaticEffect::MayPlayLandsFromGraveyardMatching(R::Land.and(historic())),
            },
        ],
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::You, amount: Value::Const(3) })],
        ..doctor("The Eighth Doctor", cost(&[generic(4), w(), u()]), 4, 4)
    }
}

// ── Companions and allies ───────────────────────────────────────────────────

/// Ace, Fearless Rebel — may sacrifice an artifact as it attacks to grow and
/// fight a defending creature.
pub fn ace_fearless_rebel() -> CardDefinition {
    companion(CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice an artifact to grow Ace and have it fight?".into(),
            filter: yours(R::Artifact),
            count: Value::ONE,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::Seq(vec![
                    plus(Selector::This, 1),
                    Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::Fight {
                            attacker: Selector::This,
                            defender: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                        }),
                    },
                ])),
            }),
            else_: None,
        })],
        ..creature("Ace, Fearless Rebel", cost(&[generic(3), g()]), vec![CreatureType::Human, CreatureType::Rebel], 2, 2)
    })
}

/// Adric, Mathematical Genius — copies an ability of yours; sacrifices to
/// counter one.
pub fn adric_mathematical_genius() -> CardDefinition {
    companion(CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                tap_cost: true,
                effect: Effect::CopyAbility {
                    what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByYou)),
                    times: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                sac_cost: true,
                effect: Effect::CounterAbility { what: target_filtered(R::HasAbilityOnStack) },
                ..Default::default()
            },
        ],
        ..creature(
            "Adric, Mathematical Genius",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            1,
        )
    })
}

/// Alistair, the Brigadier — a Soldier per historic spell; {8} on attack
/// pumps the team by your historic count.
pub fn alistair_the_brigadier() -> CardDefinition {
    let x = || Value::PermanentCountControlledByMatching(PlayerRef::You, historic());
    legend(CardDefinition {
        triggered_abilities: vec![
            on_historic_cast(make(
                token("Soldier", vec![Color::White], vec![CreatureType::Soldier], 1, 1, vec![]),
                Value::ONE,
            )),
            on_attack(Effect::MayPay {
                description: "Pay {8} to pump your creatures by your historic count?".into(),
                mana_cost: cost(&[generic(8)]),
                body: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(yours(R::Creature)),
                    power: x(),
                    toughness: x(),
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            }),
        ],
        ..creature(
            "Alistair, the Brigadier",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    })
}

/// Barbara Wright — your Sagas have read ahead.
pub fn barbara_wright() -> CardDefinition {
    companion(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Sagas you control have read ahead.",
            effect: StaticEffect::YourSagasHaveReadAhead,
        }],
        ..creature("Barbara Wright", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Advisor], 1, 3)
    })
}

/// Duggan, Private Detective — */* your hand; investigates entering and
/// attacking; one punch for twice its power.
pub fn duggan_private_detective() -> CardDefinition {
    legend(CardDefinition {
        dynamic_pt: Some(DynamicPt::ControllerHandSize),
        triggered_abilities: vec![etb(investigate(1)), on_attack(investigate(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            activate_once: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::OtherThanSource)),
                amount: Value::Times(Box::new(Value::PowerOf(Box::new(Selector::This))), Box::new(Value::Const(2))),
            },
            ..Default::default()
        }],
        ..creature(
            "Duggan, Private Detective",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Human, CreatureType::Detective],
            0,
            0,
        )
    })
}

/// Ian Chesterton — your Saga spells have replicate equal to their cost.
pub fn ian_chesterton() -> CardDefinition {
    companion(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each Saga spell you cast has replicate. The replicate cost is equal to its mana cost.",
            effect: StaticEffect::YourSpellsHaveReplicate { filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Saga) },
        }],
        ..creature("Ian Chesterton", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Scientist], 2, 3)
    })
}

/// Jamie McCrimmon — trample; pumps by each historic spell's mana value.
pub fn jamie_mccrimmon() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(Selector::TriggerSource));
    companion(CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_historic_cast(Effect::PumpPT {
            what: Selector::This,
            power: mv(),
            toughness: mv(),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Jamie McCrimmon", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2)
    })
}

/// Jo Grant — historic hand cards cycle for {2}{W}; grows per cycle.
pub fn jo_grant() -> CardDefinition {
    companion(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each historic card in your hand has cycling {2}{W}.",
            effect: StaticEffect::GrantCyclingToYourHandCards { filter: historic(), cost: cost(&[generic(2), w()]) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::YourControl),
            effect: plus(Selector::This, 1),
        }],
        ..creature("Jo Grant", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 2)
    })
}

/// K-9, Mark I — untapped, your other legends have ward {1}; makes a legend
/// unblockable.
pub fn k_9_mark_i() -> CardDefinition {
    companion(CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "As long as K-9 is untapped, other legendary creatures you control have ward {1}.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::Untapped },
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(yours(R::Creature.and(legendary())).and(R::OtherThanSource)),
                    keyword: Keyword::Ward(WardCost::generic(1)),
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: eot(target_filtered(R::Creature.and(legendary())), Keyword::Unblockable),
            ..Default::default()
        }],
        ..creature("K-9, Mark I", cost(&[u()]), vec![CreatureType::Robot, CreatureType::Dog], 1, 1)
    })
}

/// Leela, Sevateem Warrior — grows on each opponent draw past their first in
/// their draw step.
pub fn leela_sevateem_warrior() -> CardDefinition {
    companion(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl).with_filter(Predicate::Not(
                Box::new(Predicate::All(vec![
                    Predicate::CurrentStepIs(TurnStep::Draw),
                    Predicate::IsTurnOf(PlayerRef::Triggerer),
                    Predicate::ValueAtMost(Value::CardsDrawnThisStep(PlayerRef::Triggerer), Value::Const(1)),
                ])),
            )),
            effect: plus(Selector::This, 1),
        }],
        ..creature(
            "Leela, Sevateem Warrior",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    })
}

/// Nyssa of Traken — no maximum hand size; attacking, sacrifices artifacts to
/// draw and tap that many.
///
/// ⚠ Residual: the creatures are tapped up to the count on resolution, not
/// targeted.
pub fn nyssa_of_traken() -> CardDefinition {
    companion(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::SacrificeAnyNumber {
                who: PlayerRef::You,
                filter: yours(R::Artifact),
                per_each: Box::new(draw(Value::ONE)),
            },
            Effect::TapUpToValue { count: Value::SacrificedCount, filter: R::Creature, skip_untap: false, exact: false },
        ]))],
        ..creature("Nyssa of Traken", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Scientist], 3, 4)
    })
}

/// Peri Brown — historic spells have convoke.
///
/// ⚠ Residual: every historic spell, not only the first each turn.
pub fn peri_brown() -> CardDefinition {
    companion(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "The first historic spell you cast each turn has convoke.",
            effect: StaticEffect::GrantConvokeToSpells { filter: historic() },
        }],
        ..creature("Peri Brown", cost(&[generic(3), w()]), vec![CreatureType::Human], 2, 3)
    })
}

/// Romana II — copies a token that entered this turn, tapped.
pub fn romana_ii() -> CardDefinition {
    companion(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::CreateTokenCopyOf {
                extra_keywords: vec![],
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::IsToken.and(R::EnteredThisTurn)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: true,
                non_legendary: false,
                legendary: false,
            },
            ..Default::default()
        }],
        ..creature("Romana II", cost(&[generic(3), w()]), vec![CreatureType::TimeLord, CreatureType::Scientist], 3, 3)
    })
}

/// Sergeant John Benton — trample, haste; a hit draws you and that player
/// that many.
pub fn sergeant_john_benton() -> CardDefinition {
    legend(CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                draw(Value::TriggerEventAmount),
                Effect::Draw { who: Selector::Player(PlayerRef::TriggerEventPlayer), amount: Value::TriggerEventAmount },
            ]),
        }],
        ..creature(
            "Sergeant John Benton",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            4,
        )
    })
}

/// Susan Foreman — taps for {G}.
///
/// ⚠ Residual: the planeswalk replacement does nothing (no Planechase).
pub fn susan_foreman() -> CardDefinition {
    companion(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Green, Value::ONE) },
            ..Default::default()
        }],
        ..creature("Susan Foreman", cost(&[generic(1), g()]), vec![CreatureType::TimeLord], 1, 1)
    })
}

/// Tegan Jovanka — each attack, a historic attacker gets +1/+1 and
/// indestructible.
pub fn tegan_jovanka() -> CardDefinition {
    companion(CardDefinition {
        triggered_abilities: vec![on_you_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking).and(historic())),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            eot(Selector::Target(0), Keyword::Indestructible),
        ]))],
        ..creature("Tegan Jovanka", cost(&[generic(2), w()]), vec![CreatureType::Human], 2, 2)
    })
}

/// Vrestin, Menoptra Leader — X counters and X flying Insects; attacking
/// Insects grow.
pub fn vrestin_menoptra_leader() -> CardDefinition {
    legend(CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![
            etb(make(
                token(
                    "Alien Insect",
                    vec![Color::Green, Color::White],
                    vec![CreatureType::Alien, CreatureType::Insect],
                    1,
                    1,
                    vec![Keyword::Flying],
                ),
                Value::XFromCost,
            )),
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                    Predicate::AttackedWithCreatureMatching {
                        who: PlayerRef::You,
                        filter: R::HasCreatureType(CreatureType::Insect),
                    },
                ),
                effect: plus(
                    Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Insect)).and(R::IsAttacking)),
                    1,
                ),
            },
        ],
        ..creature(
            "Vrestin, Menoptra Leader",
            cost(&[x(), g(), g(), w()]),
            vec![CreatureType::Alien, CreatureType::Insect, CreatureType::Scout],
            0,
            0,
        )
    })
}

/// Displaced Dinosaurs — your historic permanents become 7/7 Dinosaurs.
///
/// ⚠ Residual: as a trigger resolves, not as the permanent enters.
pub fn displaced_dinosaurs() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(trigger_is(historic())),
            effect: Effect::BecomeCreature {
                what: Selector::TriggerSource,
                power: Value::Const(7),
                toughness: Value::Const(7),
                creature_types: vec![CreatureType::Dinosaur],
                keywords: vec![],
                duration: Duration::Permanent,
            },
        }],
        ..creature("Displaced Dinosaurs", cost(&[generic(5), g(), g()]), vec![CreatureType::Dinosaur], 7, 7)
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Ace's Baseball Bat — +3/+0, first strike attacking; equip a legend for
/// {1}.
///
/// ⚠ Residual: "must be blocked by a Dalek if able" is not modelled.
pub fn aces_baseball_bat() -> CardDefinition {
    CardDefinition {
        name: "Ace's Baseball Bat",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equip_filtered_cost: Some((R::Creature.and(legendary()), cost(&[generic(1)]))),
        equipped_bonus: Some(EquipBonus {
            power: 3,
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::IsAttacking,
                keywords: vec![Keyword::FirstStrike],
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Bessie, the Doctor's Roadster — haste Vehicle; attacking makes another
/// legend unblockable.
pub fn bessie_the_doctors_roadster() -> CardDefinition {
    CardDefinition {
        name: "Bessie, the Doctor's Roadster",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Haste, Keyword::Crew(2)],
        triggered_abilities: vec![on_attack(eot(
            target_filtered(R::Creature.and(legendary()).and(R::OtherThanSource)),
            Keyword::Unblockable,
        ))],
        ..Default::default()
    }
}

/// Five Hundred Year Diary — enters tapped; {U} per Clue; sacrifices to draw.
pub fn five_hundred_year_diary() -> CardDefinition {
    CardDefinition {
        name: "Five Hundred Year Diary",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book, ArtifactSubtype::Clue],
            ..Default::default()
        },
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(
                        Color::Blue,
                        Value::PermanentCountControlledByMatching(
                            PlayerRef::You,
                            R::HasArtifactSubtype(ArtifactSubtype::Clue),
                        ),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                sac_cost: true,
                effect: draw(Value::ONE),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Banish to Another Universe — affinity for historic permanents; exiles an
/// opponent's nonland permanent while it stays.
pub fn banish_to_another_universe() -> CardDefinition {
    CardDefinition {
        name: "Banish to Another Universe",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        affinity_filter: Some(yours(historic())),
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Gallifrey Stands — Doctors back from the graveyard; an upkeep Doctor
/// drop, and thirteen Doctors win.
pub fn gallifrey_stands() -> CardDefinition {
    CardDefinition {
        name: "Gallifrey Stands",
        cost: cost(&[generic(4), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Move {
                what: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: doctors() },
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            at(
                TurnStep::Upkeep,
                Effect::Seq(vec![
                    Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::Creature.and(doctors()),
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::PermanentCountControlledByMatching(PlayerRef::You, doctors()),
                            Value::Const(13),
                        ),
                        then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

// ── Sagas ───────────────────────────────────────────────────────────────────

/// An Unearthly Child — each chapter digs for a Doctor, a companion or a
/// Vehicle.
pub fn an_unearthly_child() -> CardDefinition {
    let dig = || Effect::RevealUntilFind {
        who: PlayerRef::You,
        find: doctors().or(R::HasKeyword(Keyword::DoctorsCompanion)).or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
        to: ZoneDest::Hand(PlayerRef::You),
        cap: Value::Const(999),
        life_per_revealed: 0,
        miss_dest: RevealMissDest::BottomRandom,
    };
    saga("An Unearthly Child", cost(&[generic(1), u(), u()]), vec![(1, dig()), (2, dig()), (3, dig())])
}

/// City of Death — a Treasure, then five copies of your non-Saga tokens.
pub fn city_of_death() -> CardDefinition {
    let copy = || Effect::CreateTokenCopyOf {
        extra_keywords: vec![],
        who: PlayerRef::You,
        count: Value::ONE,
        source: target_filtered(yours(R::IsToken).and(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga).negate())),
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
    };
    let mut chapters = vec![(1, make(treasure_token(), Value::ONE))];
    chapters.extend((2..=6).map(|n| (n, copy())));
    saga("City of Death", cost(&[generic(2), g()]), chapters)
}

/// The Caves of Androzani — stuns two tapped creatures, grows every non-Saga
/// permanent's counters twice, fetches a Doctor.
pub fn the_caves_of_androzani() -> CardDefinition {
    let grow =
        || Effect::AddOneOfAChosenCounterToEach { filter: R::Permanent.and(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga).negate()) };
    saga("The Caves of Androzani", cost(&[generic(3), w()]), vec![
        (1, Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::Tapped),
            effect: Box::new(Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::Const(2) }),
        }),
        (2, grow()),
        (3, grow()),
        (4, Effect::Search { who: PlayerRef::You, filter: doctors(), to: ZoneDest::Hand(PlayerRef::You) }),
    ])
}

/// The Curse of Fenric — kills up to one creature per player for a
/// deathtouch Mutant; blanks a creature into a 6/6; a Mutant fights it.
///
/// ⚠ Residual: II doesn't rename the creature Fenric or make it legendary;
/// III's Mutant fights any other creature.
pub fn the_curse_of_fenric() -> CardDefinition {
    let mutant = token("Mutant", vec![Color::Green], vec![CreatureType::Mutant], 3, 3, vec![Keyword::Deathtouch]);
    saga("The Curse of Fenric", cost(&[generic(2), g(), w()]), vec![
        (1, Effect::ForEachPlayerTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        count: Value::ONE,
                        definition: Arc::new(mutant),
                    },
                    Effect::Destroy { what: Selector::Target(0) },
                ])),
            }),
        }),
        (2, Effect::Seq(vec![
            Effect::LoseAllAbilities { what: target_filtered(R::Creature.and(R::NotToken)), duration: Duration::Permanent },
            Effect::SetBasePT {
                what: Selector::Target(0),
                power: Value::Const(6),
                toughness: Value::Const(6),
                duration: Duration::Permanent,
            },
        ])),
        (3, Effect::Fight {
            attacker: target_filtered(R::HasCreatureType(CreatureType::Mutant)),
            defender: Selector::TargetFiltered { slot: 1, filter: R::Creature },
        }),
    ])
}

/// The Night of the Doctor — a wrath, then a legend back with a keyword
/// counter.
pub fn the_night_of_the_doctor() -> CardDefinition {
    let kw = |k: Keyword| Effect::AddKeywordCounter { what: Selector::LastMoved, keyword: k, amount: Value::ONE };
    saga("The Night of the Doctor", cost(&[generic(4), w(), w()]), vec![
        (1, Effect::Destroy { what: Selector::EachPermanent(R::Creature) }),
        (2, Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(legendary()).and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::ChooseMode(vec![kw(Keyword::FirstStrike), kw(Keyword::Vigilance), kw(Keyword::Lifelink)]),
        ])),
    ])
}

/// The Sea Devils — two islandwalking Salamanders, then Salamander hits
/// spill onto a creature.
pub fn the_sea_devils() -> CardDefinition {
    let salamander = || {
        make(
            token(
                "Alien Salamander",
                vec![Color::Green],
                vec![CreatureType::Alien, CreatureType::Salamander],
                2,
                2,
                vec![Keyword::Landwalk(LandType::Island)],
            ),
            Value::ONE,
        )
    };
    saga("The Sea Devils", cost(&[generic(2), g()]), vec![
        (1, salamander()),
        (2, salamander()),
        (3, Effect::GrantTriggeredAbilityThisTurnToMatching {
            filter: R::HasCreatureType(CreatureType::Salamander),
            trigger: Box::new(TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                    amount: Value::TriggerEventAmount,
                },
            }),
        }),
    ])
}

/// The War Games — three goaded Warriors each, grown twice; exile a creature
/// of yours to exile every Warrior.
pub fn the_war_games() -> CardDefinition {
    let warrior = TokenDefinition {
        tapped: true,
        ..token("Warrior", vec![Color::White], vec![CreatureType::Warrior], 1, 1, vec![])
    };
    let warriors = || Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Warrior)));
    let grow = || Effect::AddCounter { what: warriors(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE };
    saga("The War Games", cost(&[generic(2), w(), w()]), vec![
        (1, Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::EachPlayer, count: Value::Const(3), definition: Arc::new(warrior) },
            Effect::GoadWhile { what: Selector::LastCreatedTokens, hold: GoadLasts::WhileSourceOnBattlefield },
        ])),
        (2, grow()),
        (3, grow()),
        (4, Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(yours(R::Creature.and(R::NotToken)))),
            then: Box::new(Effect::MayDo {
                description: "Exile a nontoken creature you control to exile all Warriors?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile {
                        what: Selector::Take {
                            inner: Box::new(Selector::EachPermanent(yours(R::Creature.and(R::NotToken)))),
                            count: Box::new(Value::ONE),
                        },
                    },
                    Effect::Exile { what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Warrior)) },
                ])),
            }),
            else_: Box::new(Effect::Noop),
        }),
    ])
}

/// Trial of a Time Lord — exiles three opposing creatures while it stays; a
/// guilty verdict bottoms them.
pub fn trial_of_a_time_lord() -> CardDefinition {
    let exile = || Effect::ExileUntilSourceLeaves {
        what: target_filtered(R::Creature.and(R::NotToken).and(R::ControlledByOpponent)),
        return_to: ExileReturnZone::Battlefield,
    };
    saga("Trial of a Time Lord", cost(&[generic(1), w(), w()]), vec![
        (1, exile()),
        (2, exile()),
        (3, exile()),
        // A tie goes to the later option, and "if guilty gets more votes"
        // makes a tie innocent.
        (4, Effect::Vote {
            options: vec![
                VoteOption::new(
                    "guilty",
                    Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
                    },
                ),
                VoteOption::new("innocent", Effect::Noop),
            ],
            tally: VoteTally::Majority,
        }),
    ])
}

// ── Instants and sorceries ──────────────────────────────────────────────────

/// Crisis of Conscience — destroy all tokens, or all nonland nontokens.
pub fn crisis_of_conscience() -> CardDefinition {
    spell(
        "Crisis of Conscience",
        cost(&[generic(4), w(), w()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::IsToken) },
            Effect::Destroy { what: Selector::EachPermanent(R::Permanent.and(R::Nonland).and(R::NotToken)) },
        ]),
    )
}

/// Reverse the Polarity — counter all other spells; switch every creature's
/// P/T; or nothing can be blocked.
///
/// ⚠ Residual: "can't be blocked" reaches only the creatures there as it
/// resolves.
pub fn reverse_the_polarity() -> CardDefinition {
    spell(
        "Reverse the Polarity",
        cost(&[generic(1), u(), u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::CounterAllOtherSpells,
            Effect::SwitchPowerToughness { what: Selector::EachPermanent(R::Creature), duration: Duration::EndOfTurn },
            eot(Selector::EachPermanent(R::Creature), Keyword::Unblockable),
        ]),
    )
}

/// The Five Doctors — up to five Doctors from library and graveyard to hand,
/// or to the battlefield kicked.
pub fn the_five_doctors() -> CardDefinition {
    let fetch = |to: ZoneDest| {
        Effect::Seq(
            (0..5)
                .map(|_| Effect::SearchZones {
                    who: PlayerRef::You,
                    zones: vec![Zone::Library, Zone::Graveyard],
                    filter: doctors(),
                    to: to.clone(),
                })
                .collect(),
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(5)]))],
        ..spell(
            "The Five Doctors",
            cost(&[generic(5), g()]),
            CardType::Sorcery,
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(fetch(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false })),
                else_: Box::new(fetch(ZoneDest::Hand(PlayerRef::You))),
            },
        )
    }
}

/// Time Lord Regeneration — a Time Lord's death reveals the next Time Lord
/// onto the battlefield.
pub fn time_lord_regeneration() -> CardDefinition {
    spell(
        "Time Lord Regeneration",
        cost(&[u()]),
        CardType::Instant,
        Effect::GrantTriggeredAbility {
            what: target_filtered(yours(R::Creature.and(time_lord()))),
            trigger: Box::new(TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::RevealUntilOneToBattlefieldRestBottom {
                    filter: R::Creature.and(time_lord()),
                    damage_controller: false,
                },
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Traverse Eternity — draw your greatest historic mana value.
pub fn traverse_eternity() -> CardDefinition {
    spell(
        "Traverse Eternity",
        cost(&[generic(2), u(), u()]),
        CardType::Sorcery,
        draw(Value::ManaValueOf(Box::new(Selector::GreatestManaValueControlledMatching {
            who: PlayerRef::You,
            filter: historic(),
        }))),
    )
}

// ── Land ────────────────────────────────────────────────────────────────────

/// Trenzalore Clocktower — {U} and a time counter; twelve of them wheel your
/// hand and graveyard back and draw seven.
pub fn trenzalore_clocktower() -> CardDefinition {
    CardDefinition {
        name: "Trenzalore Clocktower",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Blue, Value::ONE) },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Time, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Time, 12)),
                exile_self_cost: true,
                condition: Some(Predicate::SelectorExists(Selector::EachPermanent(yours(time_lord())))),
                effect: Effect::Seq(vec![
                    Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::You },
                    draw(Value::Const(7)),
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
