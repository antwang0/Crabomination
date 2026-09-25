//! Commander: the cards the **Science!** precon (PIP, Dr. Madison Li) needed
//! beyond what the catalog had. Tests in `tests/recent_b/cmdr_madison.rs`.
//!
//! Residuals (each also on its card):
//! - **C.A.M.P.** — the Junk token comes whenever the creature is colored,
//!   not only when it shares a color with the land's mana.
//! - **Endurance Bobblehead** — the X creatures are your greatest-power ones,
//!   chosen on resolution rather than targeted.
//! - **Expert-Level Safe** — both numbers are drawn at random (the
//!   equilibrium strategy); no player is asked.
//! - **Plasma Caster** — the target may be any blocking creature.
//! - **Vault 13: Dweller's Journey** — chapter I's "one per player" isn't
//!   enforced on the targets.
//! - **Vault 112: Sadistic Simulation** — chapter III reveals rather than
//!   exiles, and only a spell (not a land) may be played from among them.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EntersAsCopy, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{counter_target_spell, etb, investigate, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, r, u, w, x, Color, ManaCost};

use super::super::tap_add_colorless;

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

fn artifact_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..creature(name, mana, types, p, t) }
}

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
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

fn energy(n: i32) -> Effect {
    Effect::AddEnergy(Value::Const(n))
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn may(description: &str, body: Effect) -> Effect {
    Effect::MayDo { description: description.into(), body: Box::new(body) }
}

fn pay_energy(n: u32, then: Effect) -> Effect {
    Effect::PayEnergy { amount: n, then: Box::new(then) }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn artifact_creature_filter() -> R {
    R::Artifact.and(R::Creature)
}

fn bobbleheads() -> Value {
    Value::CountOf(Box::new(yours(R::HasArtifactSubtype(ArtifactSubtype::Bobblehead))))
}

fn any_color_mana() -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
        ..Default::default()
    }
}

fn bobblehead(name: &'static str, second: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Bobblehead], ..Default::default() },
        activated_abilities: vec![any_color_mana(), second],
        ..artifact(name, cost(&[generic(3)]))
    }
}

fn robot_token(p: i32, t: i32, tapped: bool) -> TokenDefinition {
    TokenDefinition {
        name: "Robot".into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        tapped,
        ..Default::default()
    }
}

fn make(def: TokenDefinition, count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(def) }
}

/// "Put your choice of a +1/+1, [keyword] or [keyword] counter on `what`."
fn choice_of_counter(what: Selector, plus_one: bool, keywords: &[Keyword]) -> Effect {
    let mut modes = Vec::new();
    if plus_one {
        modes.push(Effect::AddCounter { what: what.clone(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE });
    }
    for k in keywords {
        modes.push(Effect::AddKeywordCounter { what: what.clone(), keyword: k.clone(), amount: Value::ONE });
    }
    Effect::ChooseMode(modes)
}

fn upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

// ── Commander ───────────────────────────────────────────────────────────────

/// Dr. Madison Li — {U}{R}{W} Legendary Creature — Human Scientist 2/3.
/// Whenever you cast an artifact spell, you get {E}. {T}, Pay {E}: Target
/// creature gets +1/+0 and gains trample and haste until end of turn. {T},
/// Pay {E}{E}{E}: Draw a card. {T}, Pay {E}{E}{E}{E}{E}: Return target
/// artifact card from your graveyard to the battlefield tapped.
pub fn dr_madison_li() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Artifact)),
            effect: energy(1),
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                energy_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::ONE,
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
            ActivatedAbility { tap_cost: true, energy_cost: 3, effect: draw(Value::ONE), ..Default::default() },
            ActivatedAbility {
                tap_cost: true,
                energy_cost: 5,
                effect: Effect::Move {
                    what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..legendary(creature(
            "Dr. Madison Li",
            cost(&[u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            2,
            3,
        ))
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Arcade Gannon — {2}{W}{U} Legendary Creature — Human Doctor 2/3. {T}: Draw
/// a card, then discard a card. Put a quest counter on it. Once during each
/// of your turns, you may cast an artifact or Human spell from your graveyard
/// with mana value less than or equal to the number of quest counters on it.
pub fn arcade_gannon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                draw(Value::ONE),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Once each turn, cast an artifact or Human spell from your graveyard with mana value up to its quest counters.",
            effect: StaticEffect::GraveyardCastOncePerTurn {
                filter: R::Artifact.or(R::HasCreatureType(CreatureType::Human)),
                exile_after: false,
                mv_at_most_counters: Some(CounterType::Quest),
            },
        }],
        ..legendary(creature(
            "Arcade Gannon",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Doctor],
            2,
            3,
        ))
    }
}

/// Assaultron Dominator — {1}{R} Artifact Creature — Robot 2/2. When it
/// enters, you get {E}{E}. Whenever an artifact creature you control attacks,
/// you may pay {E}; if you do, put your choice of a +1/+1, first strike or
/// trample counter on it.
pub fn assaultron_dominator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(energy(2)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(trigger_is(artifact_creature_filter())),
                effect: may(
                    "Pay {E} for a counter?",
                    pay_energy(
                        1,
                        choice_of_counter(Selector::TriggerSource, true, &[Keyword::FirstStrike, Keyword::Trample]),
                    ),
                ),
            },
        ],
        ..artifact_creature("Assaultron Dominator", cost(&[generic(1), r()]), vec![CreatureType::Robot], 2, 2)
    }
}

/// Behemoth of Vault 0 — {6} Artifact Creature — Robot 6/6. Trample. When it
/// enters, you get {E}{E}{E}{E}. When it dies, you may pay {E} equal to target
/// nonland permanent's mana value; when you do, destroy that permanent.
pub fn behemoth_of_vault_0() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(energy(4)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: may(
                    "Pay energy to destroy that permanent?",
                    Effect::PayEnergyValue {
                        amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                        then: Box::new(Effect::Destroy {
                            what: target_filtered(R::Permanent.and(R::Nonland)),
                        }),
                    },
                ),
            },
        ],
        ..artifact_creature("Behemoth of Vault 0", cost(&[generic(6)]), vec![CreatureType::Robot], 6, 6)
    }
}

/// Brotherhood Scribe — {1}{W} Creature — Human Artificer 1/3. Metalcraft —
/// {T}: You get {E}; activate only with three or more artifacts. Whenever you
/// get one or more {E} during your turn, creatures you control get +1/+1 until
/// end of turn.
pub fn brotherhood_scribe() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::MetalcraftActive { who: PlayerRef::You }),
            effect: energy(1),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EnergyGained, EventScope::YourControl)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You))
                .once_per_batch(),
            effect: Effect::PumpPT {
                what: yours(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Brotherhood Scribe",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            3,
        )
    }
}

/// Curie, Emergent Intelligence — {1}{U} Legendary Artifact Creature — Robot
/// 1/3. Whenever it deals combat damage to a player, draw cards equal to its
/// base power. {1}{U}, Exile another nontoken artifact creature you control:
/// Curie becomes a copy of the exiled creature, except it has this combat
/// damage trigger.
pub fn curie_emergent_intelligence() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: draw(Value::BasePowerOf(Box::new(Selector::This))),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            exile_permanent_cost: Some((
                artifact_creature_filter().and(R::Not(Box::new(R::IsToken))).and(R::OtherThanSource),
                1,
            )),
            effect: Effect::BecomeCopyOf {
                what: Selector::This,
                source: Selector::ExiledForCost,
                extra_creature_types: vec![],
                keep_own_triggered: true,
                keep_own_activated: false,
            },
            ..Default::default()
        }],
        ..legendary(artifact_creature(
            "Curie, Emergent Intelligence",
            cost(&[generic(1), u()]),
            vec![CreatureType::Robot],
            1,
            3,
        ))
    }
}

/// Elder Owyn Lyons — {2}{W}{U} Legendary Creature — Human Knight 3/3.
/// Artifacts you control have ward {1}. When it enters or dies, return target
/// artifact card from your graveyard to your hand.
pub fn elder_owyn_lyons() -> CardDefinition {
    let back = || Effect::Move {
        what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifacts you control have ward {1}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Artifact),
                keyword: Keyword::Ward(WardCost::generic(1)),
            },
        }],
        triggered_abilities: vec![
            etb(back()),
            TriggeredAbility { event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource), effect: back() },
        ],
        ..legendary(creature(
            "Elder Owyn Lyons",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Knight],
            3,
            3,
        ))
    }
}

/// James, Wandering Dad // Follow Him — {2}{U} Legendary Creature — Human
/// Scientist 2/4. {T}: Add {C}{C}; spend this mana only to activate abilities.
/// Adventure: Follow Him {X}{U}{U} Instant — investigate X times.
pub fn james_wandering_dad() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::Const(2))),
                    crate::mana::SpendRestriction::AbilitiesOnly,
                ),
            },
            ..Default::default()
        }],
        adventure: Some(Box::new(Adventure {
            name: "Follow Him",
            cost: cost(&[x(), u(), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: Arc::new(crabomination_base::tokens::clue_token()),
            },
        })),
        ..legendary(creature(
            "James, Wandering Dad",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            2,
            4,
        ))
    }
}

/// Liberty Prime, Recharged — {2}{U}{R}{W} Legendary Artifact Creature —
/// Robot 8/8. Vigilance, trample, haste. Whenever it attacks or blocks,
/// sacrifice it unless you pay {E}{E}. {2}, {T}, Sacrifice an artifact: You
/// get {E}{E} and draw a card.
pub fn liberty_prime_recharged() -> CardDefinition {
    let upkeep_cost = || Effect::PayEnergyOrElse { amount: 2, otherwise: Box::new(Effect::SacrificeSource) };
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![
            on_attack(upkeep_cost()),
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: upkeep_cost() },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            sac_other_may_be_source: true,
            effect: Effect::Seq(vec![energy(2), draw(Value::ONE)]),
            ..Default::default()
        }],
        ..legendary(artifact_creature(
            "Liberty Prime, Recharged",
            cost(&[generic(2), u(), r(), w()]),
            vec![CreatureType::Robot],
            8,
            8,
        ))
    }
}

/// Nick Valentine, Private Eye — {2}{U} Legendary Artifact Creature — Synth
/// Detective 2/2. Can't be blocked except by artifact creatures. Whenever it
/// or another artifact creature you control dies, you may investigate.
pub fn nick_valentine_private_eye() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(artifact_creature_filter()))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(trigger_is(artifact_creature_filter())),
            effect: may("Investigate?", investigate(1)),
        }],
        ..legendary(artifact_creature(
            "Nick Valentine, Private Eye",
            cost(&[generic(2), u()]),
            vec![CreatureType::Synth, CreatureType::Detective],
            2,
            2,
        ))
    }
}

/// Paladin Danse, Steel Maverick — {2}{W} Legendary Artifact Creature — Synth
/// Knight 3/3. Vigilance, lifelink. Exile it: Each creature you control that's
/// an artifact or Human gains indestructible until end of turn.
pub fn paladin_danse_steel_maverick() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            exile_self_cost: true,
            effect: Effect::GrantKeyword {
                what: yours(R::Creature.and(R::Artifact.or(R::HasCreatureType(CreatureType::Human)))),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legendary(artifact_creature(
            "Paladin Danse, Steel Maverick",
            cost(&[generic(2), w()]),
            vec![CreatureType::Synth, CreatureType::Knight],
            3,
            3,
        ))
    }
}

/// Red Death, Shipwrecker — {U}{R} Legendary Creature — Crab Mutant 1/3.
/// Alluring Eyes — {T}: Goad target creature an opponent controls. That player
/// draws a card. You add {R}.
pub fn red_death_shipwrecker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Goad { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::ONE,
                },
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) },
            ]),
            ..Default::default()
        }],
        ..legendary(creature(
            "Red Death, Shipwrecker",
            cost(&[u(), r()]),
            vec![CreatureType::Crab, CreatureType::Mutant],
            1,
            3,
        ))
    }
}

/// Rex, Cyber-Hound — {1}{W}{U} Legendary Artifact Creature — Robot Dog 2/2.
/// Whenever it deals combat damage to a player, they mill two cards and you
/// get {E}{E}. Pay {E}{E}: Exile target creature card in a graveyard with a
/// brain counter on it; activate only as a sorcery. Rex has all activated
/// abilities of all cards in exile with brain counters on them.
pub fn rex_cyber_hound() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(2) },
                energy(2),
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 2,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::Creature.and(R::InGraveyard)), to: ZoneDest::Exile },
                Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Brain, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Rex has all activated abilities of all cards in exile with brain counters on them.",
            effect: StaticEffect::HasActivatedAbilitiesOfExiledWithCounter { counter: CounterType::Brain },
        }],
        ..legendary(artifact_creature(
            "Rex, Cyber-Hound",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Robot, CreatureType::Dog],
            2,
            2,
        ))
    }
}

/// Robobrain War Mind — {3}{U} Artifact Creature — Robot */5. Power equal to
/// the cards in your hand. When it enters, you get {E} for each artifact
/// creature you control. Whenever it attacks, you may pay {E}{E}{E}; if you
/// do, draw a card.
pub fn robobrain_war_mind() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Its power is equal to the number of cards in your hand.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: Value::HandSizeOf(PlayerRef::You),
                toughness: Value::Const(0),
            },
        }],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::CountOf(Box::new(yours(artifact_creature_filter()))))),
            on_attack(may("Pay {E}{E}{E} to draw a card?", pay_energy(3, draw(Value::ONE)))),
        ],
        ..artifact_creature("Robobrain War Mind", cost(&[generic(3), u()]), vec![CreatureType::Robot], 0, 5)
    }
}

/// Sentinel Sarah Lyons — {3}{R}{W} Legendary Creature — Human Knight 4/4.
/// Haste. As long as an artifact entered under your control this turn,
/// creatures you control get +2/+2. Battalion — whenever it and at least two
/// other creatures attack, it deals damage equal to the number of artifacts
/// you control to target player.
pub fn sentinel_sarah_lyons() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "While an artifact entered under your control this turn, creatures you control get +2/+2.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::ArtifactEnteredThisTurn { who: PlayerRef::You },
                inner: Box::new(StaticEffect::PumpPT { applies_to: yours(R::Creature), power: 2, toughness: 2 }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::AttackingWithAtLeast(3)),
            effect: Effect::DealDamage {
                to: target_filtered(R::Player),
                amount: Value::CountOf(Box::new(yours(R::Artifact))),
            },
        }],
        ..legendary(creature(
            "Sentinel Sarah Lyons",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            4,
        ))
    }
}

/// Sentry Bot — {4}{W} Artifact Creature — Robot 2/5. Flash. Costs {1} less
/// for each creature attacking you. When it enters, you get {E} for each
/// creature attacking you. At the beginning of combat on your turn, you may
/// pay {E}{E}{E}; if you do, put a +1/+1 counter on each creature you control.
pub fn sentry_bot() -> CardDefinition {
    let attackers = || Value::CreaturesAttackingPlayer(PlayerRef::You);
    CardDefinition {
        keywords: vec![Keyword::Flash],
        self_cost_reduction_per: Some((attackers(), 1)),
        triggered_abilities: vec![
            etb(Effect::AddEnergy(attackers())),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: may(
                    "Pay {E}{E}{E} for a +1/+1 counter on each creature you control?",
                    pay_energy(
                        3,
                        Effect::AddCounter { what: yours(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    ),
                ),
            },
        ],
        ..artifact_creature("Sentry Bot", cost(&[generic(4), w()]), vec![CreatureType::Robot], 2, 5)
    }
}

/// Shaun, Father of Synths — {3}{U}{R} Legendary Creature — Human Scientist
/// 3/4. Whenever you attack, you may create a tapped and attacking token copy
/// of target attacking legendary creature you control other than Shaun,
/// except it's not legendary and it's a Synth artifact creature in addition.
/// When Shaun leaves the battlefield, exile all Synth tokens you control.
pub fn shaun_father_of_synths() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: may(
                    "Create a Synth copy of an attacking legendary creature?",
                    Effect::Seq(vec![
                        Effect::CreateTokenCopyOf {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            source: target_filtered(
                                R::Creature
                                    .and(R::IsAttacking)
                                    .and(R::HasSupertype(Supertype::Legendary))
                                    .and(R::ControlledByYou)
                                    .and(R::OtherThanSource),
                            ),
                            extra_creature_types: vec![CreatureType::Synth],
                            extra_card_types: vec![CardType::Artifact],
                            override_pt: None,
                            override_colors: None,
                            enters_tapped: true,
                            non_legendary: true,
                            legendary: false,
                            extra_keywords: vec![],
                        },
                        Effect::JoinCombatAttacking { what: Selector::LastCreatedToken },
                    ]),
                ),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Exile {
                    what: yours(R::IsToken.and(R::HasCreatureType(CreatureType::Synth))),
                },
            },
        ],
        ..legendary(creature(
            "Shaun, Father of Synths",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            3,
            4,
        ))
    }
}

/// Synth Eradicator — {2}{R} Artifact Creature — Synth Soldier 3/3. Haste.
/// Whenever it attacks, exile the top card of your library. You may get
/// {E}{E}. If you don't, you may play that card this turn. {T}, Pay
/// {E}{E}{E}: It deals 3 damage to any target.
pub fn synth_eradicator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::ExileTopOfLibrary {
                who: Selector::You,
                amount: Value::ONE,
                link_to_source: false,
                face_down: false,
            },
            Effect::ChooseMode(vec![
                energy(2),
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
        ]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 3,
            effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..artifact_creature(
            "Synth Eradicator",
            cost(&[generic(2), r()]),
            vec![CreatureType::Synth, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Synth Infiltrator — {3}{U}{U} Artifact Creature — Synth 0/0. Improvise.
/// You may have it enter as a copy of any creature on the battlefield, except
/// it's a Synth artifact creature in addition to its other types.
pub fn synth_infiltrator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Improvise],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_creature_types: vec![CreatureType::Synth],
            extra_card_types: vec![CardType::Artifact],
            ..Default::default()
        }),
        ..artifact_creature("Synth Infiltrator", cost(&[generic(3), u(), u()]), vec![CreatureType::Synth], 0, 0)
    }
}

/// The Motherlode, Excavator — {3}{R}{R} Legendary Artifact Creature — Robot
/// 5/5. When it enters, choose target opponent; you get {E} for each nonbasic
/// land that player controls. Whenever it attacks, you may pay {E}{E}{E}{E};
/// when you do, destroy target nonbasic land defending player controls, and
/// creatures that player controls without flying can't block this turn.
pub fn the_motherlode_excavator() -> CardDefinition {
    let nonbasic = || R::Land.and(R::Not(Box::new(R::IsBasicLand)));
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::AddEnergy(Value::CountOf(Box::new(Selector::ControlledBy {
                    who: PlayerRef::Target(0),
                    filter: nonbasic(),
                })))),
            }),
            on_attack(may(
                "Pay {E}{E}{E}{E} to destroy a nonbasic land?",
                pay_energy(
                    4,
                    Effect::Seq(vec![
                        Effect::Destroy { what: target_filtered(nonbasic().and(R::ControlledByDefendingPlayer)) },
                        Effect::GrantKeyword {
                            what: Selector::EachPermanent(
                                R::Creature
                                    .and(R::ControlledByDefendingPlayer)
                                    .and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                            ),
                            keyword: Keyword::CantBlock,
                            duration: Duration::EndOfTurn,
                        },
                    ]),
                ),
            )),
        ],
        ..legendary(artifact_creature(
            "The Motherlode, Excavator",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Robot],
            5,
            5,
        ))
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Automated Assembly Line — {1}{W} Artifact. Whenever one or more artifact
/// creatures you control deal combat damage to a player, you get {E}. Pay
/// {E}{E}{E}: Create a tapped 3/3 colorless Robot artifact creature token.
pub fn automated_assembly_line() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(trigger_is(artifact_creature_filter()))
                .once_per_batch(),
            effect: energy(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 3,
            effect: make(robot_token(3, 3, true), Value::ONE),
            ..Default::default()
        }],
        ..artifact("Automated Assembly Line", cost(&[generic(1), w()]))
    }
}

/// Brotherhood Vertibird — {3} Artifact — Vehicle */4. Flying. Its power is
/// the number of artifacts you control. Crew 2.
pub fn brotherhood_vertibird() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower { base_p: 0, base_t: 4 }),
        ..artifact("Brotherhood Vertibird", cost(&[generic(3)]))
    }
}

/// C.A.M.P. — {3} Artifact — Fortification. Whenever fortified land is tapped
/// for mana, put a +1/+1 counter on target creature you control; if it shares
/// a color with the mana, create a Junk token. Fortify {3}.
///
/// ⚠ Residual: the Junk token comes whenever the creature is colored.
pub fn c_a_m_p() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Fortification], ..Default::default() },
        keywords: vec![Keyword::Fortify(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::TappedForMana, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::Target(0),
                            filter: R::Not(Box::new(R::Colorless)),
                        },
                        then: Box::new(make(crabomination_base::tokens::junk_token(), Value::ONE)),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            }],
            ..Default::default()
        }),
        ..artifact("C.A.M.P.", cost(&[generic(3)]))
    }
}

/// Endurance Bobblehead — {3} Artifact — Bobblehead. {T}: Add one mana of any
/// color. {3}, {T}: Up to X target creatures you control get +1/+0 and gain
/// indestructible until end of turn, X the Bobbleheads you control. Sorcery.
///
/// ⚠ Residual: the creatures are your greatest-power ones, chosen on
/// resolution.
pub fn endurance_bobblehead() -> CardDefinition {
    let chosen = || Selector::TakeGreatestPower { inner: Box::new(yours(R::Creature)), count: Box::new(bobbleheads()) };
    bobblehead(
        "Endurance Bobblehead",
        ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: chosen(), power: Value::ONE, toughness: Value::Const(0), duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: chosen(), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        },
    )
}

/// Intelligence Bobblehead — {3} Artifact — Bobblehead. {T}: Add one mana of
/// any color. {5}, {T}: Draw X cards, X the Bobbleheads you control.
pub fn intelligence_bobblehead() -> CardDefinition {
    bobblehead(
        "Intelligence Bobblehead",
        ActivatedAbility { mana_cost: cost(&[generic(5)]), tap_cost: true, effect: draw(bobbleheads()), ..Default::default() },
    )
}

/// Expert-Level Safe — {2} Artifact. When it enters, exile the top two cards
/// of your library face down. {1}, {T}: You and target opponent each secretly
/// choose 1, 2, or 3. If they match, sacrifice it and put all cards exiled
/// with it into their owners' hands. Otherwise, exile the top card of your
/// library face down.
///
/// ⚠ Residual: both numbers are drawn at random.
pub fn expert_level_safe() -> CardDefinition {
    let stash = |n| Effect::ExileTopOfLibrary {
        who: Selector::You,
        amount: Value::Const(n),
        link_to_source: true,
        face_down: true,
    };
    CardDefinition {
        triggered_abilities: vec![etb(stash(2))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::SecretNumbersMatch {
                    opponent: PlayerRef::Target(0),
                    max: 3,
                    on_match: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::CardExiledWithSource,
                            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                        },
                        Effect::SacrificeSource,
                    ])),
                    on_miss: Box::new(stash(1)),
                }),
            },
            ..Default::default()
        }],
        ..artifact("Expert-Level Safe", cost(&[generic(2)]))
    }
}

/// Nuka-Cola Vending Machine — {3} Artifact. {1}, {T}: Create a Food token.
/// Whenever you sacrifice a Food, create a tapped Treasure token.
pub fn nuka_cola_vending_machine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: make(crabomination_base::tokens::food_token(), Value::ONE),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(trigger_is(R::HasArtifactSubtype(ArtifactSubtype::Food))),
            effect: make(TokenDefinition { tapped: true, ..crabomination_base::tokens::treasure_token() }, Value::ONE),
        }],
        ..artifact("Nuka-Cola Vending Machine", cost(&[generic(3)]))
    }
}

/// Plasma Caster — {1}{R} Artifact — Equipment. Equipped creature gets +1/+1.
/// Whenever it attacks, you get {E}{E}. Pay {E}{E}: Choose target creature
/// blocking equipped creature; flip a coin: win, exile it; lose, 1 damage to
/// it. Equip {2}.
///
/// ⚠ Residual: the target may be any blocking creature.
pub fn plasma_caster() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_attack(energy(2))],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 2,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Exile { what: target_filtered(R::Creature.and(R::IsBlocking)) }),
                on_tails: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::ONE }),
            },
            ..Default::default()
        }],
        ..artifact("Plasma Caster", cost(&[generic(1), r()]))
    }
}

/// T-45 Power Armor — {2} Artifact — Equipment. When it enters, you get
/// {E}{E}. Equipped creature gets +3/+3 and doesn't untap during its
/// controller's untap step. At the beginning of your upkeep, you may pay {E};
/// if you do, untap equipped creature, then put your choice of a menace,
/// trample, or lifelink counter on it. Equip {3}.
pub fn t_45_power_armor() -> CardDefinition {
    let wearer = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 3, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Equipped creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: wearer() },
        }],
        triggered_abilities: vec![
            etb(energy(2)),
            TriggeredAbility {
                event: upkeep().with_filter(Predicate::SelectorExists(wearer())),
                effect: may(
                    "Pay {E} to untap the equipped creature and give it a counter?",
                    pay_energy(
                        1,
                        Effect::Seq(vec![
                            Effect::Untap { what: wearer(), up_to: None },
                            choice_of_counter(wearer(), false, &[Keyword::Menace, Keyword::Trample, Keyword::Lifelink]),
                        ]),
                    ),
                ),
            },
        ],
        ..artifact("T-45 Power Armor", cost(&[generic(2)]))
    }
}

/// The Prydwen, Steel Flagship — {4}{W}{W} Legendary Artifact — Vehicle 6/6.
/// Flying. Whenever another nontoken artifact you control enters, create a
/// 2/2 white Human Knight token with "This token gets +2/+2 as long as an
/// artifact entered the battlefield under your control this turn." Crew 2.
pub fn the_prydwen_steel_flagship() -> CardDefinition {
    let knight = TokenDefinition {
        name: "Human Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This token gets +2/+2 as long as an artifact entered under your control this turn.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::ArtifactEnteredThisTurn { who: PlayerRef::You },
                inner: Box::new(StaticEffect::PumpPT { applies_to: Selector::This, power: 2, toughness: 2 }),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(trigger_is(
                R::Artifact.and(R::Not(Box::new(R::IsToken))).and(R::OtherThanSource),
            )),
            effect: make(knight, Value::ONE),
        }],
        ..artifact("The Prydwen, Steel Flagship", cost(&[generic(4), w(), w()]))
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Nerd Rage — {2}{U} Enchantment — Aura. When it enters, draw two cards.
/// Enchanted creature has "You have no maximum hand size" and "Whenever this
/// creature attacks, if you have ten or more cards in hand, it gets +10/+10
/// until end of turn."
pub fn nerd_rage() -> CardDefinition {
    CardDefinition {
        name: "Nerd Rage",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![
            etb(draw(Value::Const(2))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource).with_filter(
                    Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::Const(10)),
                ),
                effect: Effect::PumpPT {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    power: Value::Const(10),
                    toughness: Value::Const(10),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Overencumbered — {1}{W} Enchantment — Aura. Enchant opponent. When it
/// enters, enchanted opponent creates a Clue, a Food, and a Junk token. At the
/// beginning of combat on enchanted opponent's turn, that player may pay {1}
/// for each artifact they control; if they don't, creatures can't attack this
/// combat.
pub fn overencumbered() -> CardDefinition {
    let gift = |def: TokenDefinition| Effect::CreateToken {
        who: PlayerRef::EnchantedPlayer,
        count: Value::ONE,
        definition: Arc::new(def),
    };
    CardDefinition {
        name: "Overencumbered",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::OpponentPlayer) },
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                gift(crabomination_base::tokens::clue_token()),
                gift(crabomination_base::tokens::food_token()),
                gift(crabomination_base::tokens::junk_token()),
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::EnchantedPlayer)),
                effect: Effect::EnchantedPlayerPaysPerArtifactOrNoAttacks,
            },
        ],
        ..Default::default()
    }
}

/// Vault 13: Dweller's Journey — {3}{W} Enchantment — Saga. I — For each
/// player, exile up to one other target enchantment or creature that player
/// controls until this Saga leaves the battlefield. II — You gain 2 life and
/// scry 2. III — Return two cards exiled with this Saga to the battlefield
/// under their owners' control and put the rest on the bottom of their
/// owners' libraries.
///
/// ⚠ Residual: chapter I's "one per player" isn't enforced on the targets.
pub fn vault_13_dwellers_journey() -> CardDefinition {
    saga("Vault 13: Dweller's Journey", cost(&[generic(3), w()]), vec![
        (
            1,
            Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 4,
                filter: R::Creature.or(R::Enchantment).and(R::OtherThanSource),
                effect: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::Target(0),
                    return_to: crate::card::ExileReturnZone::Battlefield,
                }),
            },
        ),
        (
            2,
            Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            ]),
        ),
        (3, Effect::ReturnSomeExiledWithSourceRestToBottom { count: 2 }),
    ])
}

/// Vault 112: Sadistic Simulation — {2}{U}{R} Enchantment — Saga. I, II — Tap
/// up to one target creature and put a stun counter on it; you get {E}{E}.
/// III — Pay any amount of {E}. If you paid one or more, shuffle your library,
/// then exile that many cards from the top; you may play one of those cards
/// without paying its mana cost.
///
/// ⚠ Residual: chapter III reveals rather than exiles, and only a spell may be
/// played from among them.
pub fn vault_112_sadistic_simulation() -> CardDefinition {
    let stun = || {
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::Target(0) },
                    Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                ])),
            },
            energy(2),
        ])
    };
    saga("Vault 112: Sadistic Simulation", cost(&[generic(2), u(), r()]), vec![
        (1, stun()),
        (2, stun()),
        (
            3,
            Effect::PayAnyEnergy {
                then: Box::new(Effect::If {
                    cond: Predicate::ValueAtLeast(Value::EnergyPaidThisEffect, Value::ONE),
                    then: Box::new(Effect::Seq(vec![
                        Effect::ShuffleLibrary { who: PlayerRef::You },
                        Effect::RevealTopMayCastOneFree {
                            count: Value::EnergyPaidThisEffect,
                            max_mv: Value::Const(99),
                            filter: None,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ),
    ])
}

// ── Instants and lands ──────────────────────────────────────────────────────

/// Bottle-Cap Blast — {4}{R} Instant. Improvise. 5 damage to any target; excess
/// damage dealt to a permanent this way makes that many tapped Treasures.
pub fn bottle_cap_blast() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Improvise],
        ..spell(
            "Bottle-Cap Blast",
            cost(&[generic(4), r()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(5) },
                make(
                    TokenDefinition { tapped: true, ..crabomination_base::tokens::treasure_token() },
                    Value::ExcessDamageDealtThisResolution,
                ),
            ]),
        )
    }
}

/// Electrosiphon — {U}{U}{R} Instant. Counter target spell. You get {E} equal
/// to its mana value.
pub fn electrosiphon() -> CardDefinition {
    spell(
        "Electrosiphon",
        cost(&[u(), u(), r()]),
        CardType::Instant,
        Effect::WithX {
            x: Value::ManaValueOf(Box::new(Selector::Target(0))),
            body: Box::new(Effect::Seq(vec![counter_target_spell(), Effect::AddEnergy(Value::XFromCost)])),
        },
    )
}

/// HELIOS One — Land. {T}: Add {C}. {1}, {T}: You get {E}. {3}, {T}, Pay X
/// {E}, Sacrifice it: Destroy target nonland permanent with mana value X.
/// Activate only as a sorcery.
pub fn helios_one() -> CardDefinition {
    CardDefinition {
        name: "HELIOS One",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility { mana_cost: cost(&[generic(1)]), tap_cost: true, effect: energy(1), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_cost: true,
                energy_x_cost: true,
                sorcery_speed: true,
                effect: Effect::Destroy {
                    what: target_filtered(R::Permanent.and(R::Nonland).and(R::ManaValueExactlyXFromCost)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
