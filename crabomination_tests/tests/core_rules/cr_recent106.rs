//! CR 508.4 — a creature put onto the battlefield attacking was never
//! declared as an attacker, so "whenever ~ attacks" abilities don't trigger
//! for it. General Kreat's "whenever a Goblin you control attacks, create a
//! Goblin that's tapped and attacking" used to fire again for each token it
//! made: 922 Goblins on one seat by turn 57 of a 15-seat pod game.
//!
//! CR 102.2 — "an opponent controls four or more nonbasic lands" asks each
//! opponent on their own (`Predicate::AnOpponentControlsAtLeast`).
//!
//! CR 506.3 — "creatures attacking you" are those whose defender is you, not
//! every attacker at the table (`SelectionRequirement::IsAttackingYou`).
//!
//! CR 701.14 — "each of those tokens fights a different one of those
//! creatures" (`Effect::CreateTokensToFightEach`, Ezuri's Predation).
//!
//! CR 903.3 — "the greatest mana value among your commanders" reads every
//! commander in every zone (`Value::GreatestCommanderManaValue`).
//!
//! CR 603.2c — "whenever one or more creatures deal combat damage to you" is
//! one event per damage step, and its filter applies: the combat hook for
//! `ControllerDealtCombatDamage` fired once per dealer with both dropped.
//!
//! CR 800.4 — a value that reads ONE player can't be handed "each opponent":
//! `resolve_player` answers the first seat and drops the rest (a 21-seat
//! debug pod aborted on Phyrexian Swarmlord's `PoisonCountersOf(EachOpponent)`).
//! The same holds for a predicate's `who`; "if an opponent …" is
//! `Predicate::ForAnyPlayer`.
//!
//! CR 608.2 — a per-player loop whose body asks a prompting seat parks the
//! rest of that body; it must resume bound to the same player. Six loops
//! resumed it under the stack item's context (`ForEachOpponent` lost its
//! `Triggerer`, five others their seat as controller).
//!
//! CR 205.4e — a legendary instant or sorcery needs a legendary creature or
//! planeswalker under its caster's control (`legendary_spell_castable`).

use crabomination::card::SelectionRequirement;
use crabomination::catalog;
use crabomination::effect::Predicate;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

#[test]
fn cr_508_4_a_token_entering_attacking_does_not_trigger_attacks() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let kreat = g.add_card_to_battlefield(0, catalog::general_kreat_the_boltbringer());
    g.clear_sickness(kreat);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kreat,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let goblins: Vec<_> =
        g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Goblin").map(|c| c.id).collect();
    assert_eq!(goblins.len(), 1, "one declared Goblin, one token");
    let tok = goblins[0];
    assert!(g.battlefield_find(tok).unwrap().tapped);
    assert!(g.attacking().iter().any(|a| a.attacker == tok), "still an attacking creature");
}

#[test]
fn cr_102_2_an_opponent_controls_at_least_counts_per_opponent() {
    let mut g = multi_player_game(4);
    let pred = Predicate::AnOpponentControlsAtLeast { filter: SelectionRequirement::IsNonbasicLand, n: 4 };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    // Three opponents with two nonbasics each: six across the table, four
    // under no one.
    for seat in 1..4 {
        for _ in 0..2 {
            g.add_card_to_battlefield(seat, catalog::tundra());
        }
    }
    // Basics and your own nonbasics don't count.
    for _ in 0..4 {
        g.add_card_to_battlefield(1, catalog::plains());
        g.add_card_to_battlefield(0, catalog::tundra());
    }
    assert!(!g.evaluate_predicate(&pred, &ctx), "summed across the table, not per opponent");
    for _ in 0..2 {
        g.add_card_to_battlefield(2, catalog::tundra());
    }
    assert!(g.evaluate_predicate(&pred, &ctx), "one opponent has four");
}

#[test]
fn cr_506_3_attacking_you_is_not_attacking_someone_else() {
    use crabomination::card::Value;
    use crabomination::effect::Selector;
    let mut g = multi_player_game(3);
    g.active_player_idx = 1;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.set_attacking(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(2) },
    ]);
    let count = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::IsAttackingYou)),
        filter: SelectionRequirement::Creature,
    };
    for seat in [0, 2] {
        let ctx = EffectContext::for_spell(seat, None, 0, 0);
        let one = Predicate::ValueEquals(count.clone(), Value::Const(1));
        assert!(g.evaluate_predicate(&one, &ctx), "seat {seat} is attacked by one");
    }
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let none = Predicate::ValueEquals(count, Value::Const(0));
    assert!(g.evaluate_predicate(&none, &ctx), "the attacker is attacked by none");
}

#[test]
fn cr_701_14_each_token_fights_a_different_creature() {
    use crabomination::card::{CardType, TokenDefinition};
    use crabomination::effect::Effect;
    use std::sync::Arc;
    let mut g = multi_player_game(3);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(2, catalog::hill_giant());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let beast = Arc::new(TokenDefinition {
        name: "Phyrexian Beast".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        ..Default::default()
    });
    let effect = Effect::CreateTokensToFightEach {
        filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
        definition: beast,
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&effect, &ctx).expect("resolve");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "4 damage kills the Bears");
    assert!(g.battlefield_find(giant).is_none(), "and the 3/3 Giant");
    assert!(g.battlefield_find(mine).is_some(), "your own creatures aren't prey");
    let beasts: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Beast").collect();
    assert_eq!(beasts.len(), 2, "one per opposing creature");
    let damage: Vec<u32> = beasts.iter().map(|c| c.damage).collect();
    assert!(damage.contains(&2) && damage.contains(&3), "each fought a different one: {damage:?}");
}

#[test]
fn cr_903_3_greatest_commander_mana_value_reads_every_zone() {
    use crabomination::card::Value;
    use crabomination::effect::PlayerRef;
    let mut g = two_player_game();
    let v = Value::GreatestCommanderManaValue(PlayerRef::You);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let is = |g: &GameState, n| g.evaluate_predicate(&Predicate::ValueEquals(v.clone(), Value::Const(n)), &ctx);
    assert!(is(&g, 0), "no commander");
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.players[0].commanders.extend([bears, angel]);
    assert!(is(&g, 5), "the Angel in the graveyard still counts");
}

#[test]
fn cr_603_2c_damage_to_you_triggers_once_per_swing() {
    use crabomination::card::{
        CardDefinition, CardType, EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility, Value,
    };
    use crabomination::effect::{Effect, Selector};
    // "Whenever one or more creatures an opponent controls deal combat damage
    // to you, you gain 1 life."
    let ward = CardDefinition {
        name: "Batch Ward (test)",
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::ControlledByOpponent,
                })
                .once_per_batch(),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    };
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, ward);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, 20 - 4 + 1, "both connected; one trigger for the swing");
}

/// The `Value` variants whose evaluator reads a single player
/// (`resolve_player`). Given a multi-player ref they silently read one seat.
const SINGLE_PLAYER_VALUES: &[&str] = &[
    "ArtifactsEnteredThisTurn",
    "CardTypesInGraveyard",
    "CardsDrawnThisStep",
    "CardsDrawnThisTurn",
    "CreatureCountControlledBy",
    "CreaturesDiedThisTurn",
    "DistinctManaValuesInGraveyard",
    "DistinctPowersAmongCreaturesControlled",
    "DistinctTwoColorPairsControlled",
    "DomainCount",
    "GreatestManaValueAmongPermanents",
    "GreatestManaValueInGraveyard",
    "GreatestToxicAmongControlled",
    "HalfLibrarySizeRoundedUp",
    "HalfLifeRoundedUp",
    "LandsPlayedThisTurn",
    "LibrarySizeOf",
    "LifeOf",
    "MountsVehiclesEnteredThisTurn",
    "MulticoloredSpellsCastThisTurn",
    "NonbasicLandCountControlledBy",
    "PermanentCountControlledBy",
    "PermanentsSacrificedThisTurn",
    "PlayerSpeed",
    "PoisonCountersOf",
    "SnowPermanentCountControlledBy",
    "UnlockedDoorsControlled",
];
/// The `Predicate` variants whose `who` field is read as a single player.
const SINGLE_PLAYER_PREDICATES: &[&str] = &[
    "ActivatedLoyaltyThisTurn",
    "AnotherCreatureEnteredThisTurn",
    "ArtifactEnteredThisTurn",
    "AttackedDefenderWithCountAtLeast",
    "AttackedWithCountAtLeast",
    "AttackedWithCreatureMatching",
    "AttackedWithTotalPowerAtLeast",
    "CelebrationActive",
    "CommittedCrimeThisTurn",
    "ControlsEachGreatestPowerCreature",
    "ControlsGreatestPowerCreature",
    "ControlsOutlaw",
    "CorruptedActive",
    "CovenActive",
    "CreatureEnteredThisTurn",
    "CreaturesCastThisTurnAtLeast",
    "CreaturesDiedThisTurnAtLeast",
    "DeliriumActive",
    "DescendActive",
    "DescendedThisTurn",
    "DistinctCounterKindsAmongCreaturesAtLeast",
    "DistinctUnlockedDoorNamesAtLeast",
    "FaceDownActivityThisTurn",
    "FerociousActive",
    "FirstLifeGainThisTurn",
    "FormidableActive",
    "HellbentActive",
    "InstantsOrSorceriesCastThisTurnAtLeast",
    "LifeGainedThisTurnAtLeast",
    "MetalcraftActive",
    "NoSpellCastFromHandThisTurn",
    "NoncreatureSpellsCastThisTurnAtLeast",
    "OilActivityThisTurn",
    "OwnsSourceNamedCardInEveryZone",
    "PermanentsSacrificedThisTurnAtLeast",
    "PlaneswalkerEnteredThisTurn",
    "PlayerIsOpponent",
    "RevoltActive",
    "SacrificedArtifactThisTurn",
    "SpellsCastThisTurnAtLeast",
    "SpellsCastThisTurnEquals",
    "ThresholdActive",
    "UnlockedDoorsControlledAtLeast",
    "VoidActive",
];
const MULTI_PLAYER_REFS: &[&str] =
    &["EachOpponent", "EachPlayer", "EachTeammate", "EachOpponentExceptTriggerer", "EachPlayerWithoutMaxSpeed"];

fn single_player_value_misuse(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, inner) in map {
                if SINGLE_PLAYER_VALUES.contains(&k.as_str())
                    && let serde_json::Value::String(r) = inner
                    && MULTI_PLAYER_REFS.contains(&r.as_str())
                {
                    out.push(format!("{k}({r})"));
                }
                if SINGLE_PLAYER_PREDICATES.contains(&k.as_str())
                    && let Some(serde_json::Value::String(r)) = inner.get("who")
                    && MULTI_PLAYER_REFS.contains(&r.as_str())
                {
                    out.push(format!("{k} {{ who: {r} }}"));
                }
                single_player_value_misuse(inner, out);
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(|i| single_player_value_misuse(i, out)),
        _ => {}
    }
}

#[test]
fn cr_800_4_no_single_player_value_or_predicate_reads_each_opponent() {
    let mut bad = Vec::new();
    for factory in catalog::all_known_factories() {
        let def = factory();
        let json = serde_json::to_value(&def).expect("definition serializes");
        let mut hits = Vec::new();
        single_player_value_misuse(&json, &mut hits);
        for h in hits {
            bad.push(format!("{}: {h}", def.name));
        }
    }
    bad.sort();
    bad.dedup();
    assert!(bad.is_empty(), "single-player values given a multi-player ref ({}): {bad:#?}", bad.len());
}

/// CR 800.4 — "each opponent loses 1 life for each creature *they* control"
/// is per opponent: Netherborn Phalanx used to charge everyone the first
/// opponent's creature count.
#[test]
fn cr_800_4_netherborn_phalanx_charges_each_opponent_their_own_count() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    let id = g.add_card_to_battlefield_entering(0, catalog::netherborn_phalanx());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 2);
    assert_eq!(g.players[2].life, l2, "no creatures, no loss");
}

/// CR 800.4 — "if an opponent cast three or more spells this turn" asks every
/// opponent: Mindbreak Trap's alternative cost used to read only the first.
#[test]
fn cr_800_4_for_any_player_asks_every_opponent() {
    use crabomination::card::Value;
    use crabomination::effect::PlayerRef;
    let mut g = multi_player_game(3);
    let pred = Predicate::ForAnyPlayer {
        who: PlayerRef::EachOpponent,
        pred: Box::new(Predicate::SpellsCastThisTurnAtLeast { who: PlayerRef::Triggerer, at_least: Value::Const(3) }),
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    assert!(!g.evaluate_predicate(&pred, &ctx));
    g.players[2].spells_cast_this_turn = 3;
    assert!(g.evaluate_predicate(&pred, &ctx), "the second opponent counts");
}

#[test]
fn cr_608_2_a_parked_per_opponent_body_resumes_bound_to_its_opponent() {
    use crabomination::decision::{Decision, DecisionAnswer};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[1].poison_counters = 3;
    let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_graveyard(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::geths_summons());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].wants_ui = true;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let mut asked = 0;
    for _ in 0..12 {
        let Some(p) = g.pending_decision.as_ref() else {
            if g.stack.is_empty() {
                break;
            }
            g.perform_action(GameAction::PassPriority).expect("pass");
            continue;
        };
        let answer = match &p.decision {
            Decision::ChooseCards { candidates, .. } => {
                asked += 1;
                DecisionAnswer::Cards(candidates.iter().take(1).map(|(id, _)| *id).collect())
            }
            other => panic!("unexpected ask {other:?}"),
        };
        g.perform_action(GameAction::SubmitDecision(answer)).expect("answer");
    }
    g.players[0].wants_ui = false;
    assert_eq!(asked, 2, "one pick from each graveyard");
    assert_eq!(g.battlefield_find(mine).map(|c| c.controller), Some(0));
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0), "the corrupted opponent's Angel");
}

/// CR 603.7c — a delayed trigger's X is the X its creating effect had
/// ("reveal until you reveal that many creature cards"), not the zero a
/// fire-time context carries. `Effect::RevealUntilMatchingToBattlefield`
/// puts every hit onto the battlefield and shuffles the misses back.
#[test]
fn cr_603_7c_a_delayed_trigger_keeps_its_x() {
    use crabomination::card::{CardDefinition, CardType, SelectionRequirement as R};
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "Delayed Destiny",
        card_types: vec![CardType::Sorcery],
        effect: Effect::WithX {
            x: Value::CountOf(Box::new(yours())),
            body: Box::new(Effect::Seq(vec![
                Effect::Exile { what: yours() },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::RevealUntilMatchingToBattlefield {
                        filter: R::Creature,
                        count: Value::XFromCost,
                        rest_bottom: false,
                    }),
                },
            ])),
        },
        ..Default::default()
    });
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for def in [catalog::forest(), catalog::grizzly_bears(), catalog::forest(), catalog::grizzly_bears(), catalog::grizzly_bears()] {
        g.add_card_to_library(0, def);
    }
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 0, "both exiled");
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "two exiled, so two creature cards come back — not zero, not three");
    assert_eq!(g.players[0].library.len(), 3, "the two Forests and the third Bears are shuffled back");
}

/// Answer every `ChooseCards` ask by taking the first candidate; returns how
/// many were asked.
fn answer_first_cards(g: &mut GameState) -> usize {
    use crabomination::decision::{Decision, DecisionAnswer};
    let mut asked = 0;
    for _ in 0..12 {
        let Some(p) = g.pending_decision.as_ref() else {
            if g.stack.is_empty() {
                break;
            }
            g.perform_action(GameAction::PassPriority).expect("pass");
            continue;
        };
        let answer = match &p.decision {
            Decision::ChooseCards { candidates, .. } => {
                asked += 1;
                DecisionAnswer::Cards(candidates.iter().take(1).map(|(id, _)| *id).collect())
            }
            other => panic!("unexpected ask {other:?}"),
        };
        g.perform_action(GameAction::SubmitDecision(answer)).expect("answer");
    }
    asked
}

/// CR 608.2 — a body parked for a prompt resumes with the X its `WithX` bound
/// (Kodama of the East Tree resumed at X = 0 in a pod, matched nothing and
/// leaked its answer).
#[test]
fn cr_608_2_a_parked_with_x_body_keeps_its_x() {
    use crabomination::card::{CardDefinition, CardType, SelectionRequirement as R};
    use crabomination::effect::{Effect, PlayerRef, Value};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "Measured Arrival",
        card_types: vec![CardType::Sorcery],
        effect: Effect::WithX {
            x: Value::Const(3),
            body: Box::new(Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::PermanentCard.and(R::ManaValueAtMostXFromCost),
                count: Value::Const(1),
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            }),
        },
        ..Default::default()
    });
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::serra_angel());
    g.players[0].wants_ui = true;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    let asked = answer_first_cards(&mut g);
    g.players[0].wants_ui = false;
    assert_eq!(asked, 1);
    assert!(g.battlefield_find(bear).is_some(), "the Bears (MV 2 <= 3) came down on resume");
}

/// CR 608.2 — `Effect::AsPlayer` asks the chosen player, and their parked
/// pick resumes as them: the card goes to *their* hand from *their* graveyard.
#[test]
fn cr_608_2_a_parked_as_player_body_resumes_as_that_player() {
    use crabomination::card::{CardDefinition, CardType, SelectionRequirement as R};
    use crabomination::effect::{Effect, PlayerRef, Value};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "Gift of Memory",
        card_types: vec![CardType::Sorcery],
        effect: Effect::AsPlayer {
            who: PlayerRef::Seat(1),
            body: Box::new(Effect::ReturnGraveyardCardsToHand { filter: R::Any, max: Value::Const(1) }),
        },
        ..Default::default()
    });
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_graveyard(1, catalog::serra_angel());
    g.players[1].wants_ui = true;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    let asked = answer_first_cards(&mut g);
    g.players[1].wants_ui = false;
    assert_eq!(asked, 1);
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs));
}

/// CR 702.74a — an evoked creature's controller *sacrifices* it: its
/// leave-the-battlefield trigger fires (it used to be moved to the
/// graveyard, outside the sacrifice funnel, and the trigger was lost).
#[test]
fn cr_702_74a_an_evoked_creature_is_sacrificed() {
    use crabomination::card::{CardDefinition, CardType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let evoker = g.add_card_to_hand(0, CardDefinition {
        name: "Parting Gift",
        cost: crabomination::mana::cost(&[crabomination::mana::generic(5)]),
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        alternative_cost: Some(crabomination::effect::shortcut::evoke(crabomination::mana::cost(&[crabomination::mana::generic(1)]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        }],
        ..Default::default()
    });
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: evoker, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("evoke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(evoker).is_none());
    assert_eq!(g.players[0].life, 23, "the leave trigger fired");
}

/// CR 205.4e — a legendary sorcery may be cast only while its caster controls
/// a legendary creature or planeswalker (Jaya's Immolating Inferno); a
/// nonlegendary creature does not count.
#[test]
fn cr_205_4e_a_legendary_sorcery_needs_a_legendary_creature_or_planeswalker() {
    use crabomination::card::{CardDefinition, CardType, Supertype};
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let spell = || CardDefinition {
        name: "Legendary Insight",
        cost: crabomination::mana::cost(&[crabomination::mana::generic(1)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ..Default::default()
    };
    let cast = |g: &mut GameState, id| {
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
    };
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let first = g.add_card_to_hand(0, spell());
    assert!(cast(&mut g, first).is_err(), "a nonlegendary creature is not enough");
    g.players[0].mana_pool = Default::default();
    g.add_card_to_battlefield(0, catalog::isamaru_hound_of_konda());
    cast(&mut g, first).expect("a legendary creature lets it be cast");
    drain_stack(&mut g);
}

/// CR 608.2 — once every seat has answered a table choice, its replay log is
/// spent. Dredge the Mire and Explosion of Riches left their answers behind,
/// which a strict debug pod caught (seed 9291). Nextest runs each test in its
/// own process, so the strict leak check (read once) is switched on here.
#[test]
fn cr_608_2_table_choices_spend_their_answer_log() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    // SAFETY: set before any thread reads the environment.
    unsafe { std::env::set_var("CRAB_ANSWER_LOG", "strict") };
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    for seat in 0..3 {
        g.add_card_to_library(seat, catalog::plains());
        g.add_card_to_graveyard(seat, catalog::grizzly_bears());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    for spell in [catalog::explosion_of_riches(), catalog::dredge_the_mire()] {
        let id = g.add_card_to_hand(0, spell);
        for c in [crabomination::mana::Color::Blue, crabomination::mana::Color::Black, crabomination::mana::Color::Red] {
            g.players[0].mana_pool.add(c, 10);
        }
        g.players[0].mana_pool.add_colorless(10);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
    }
}

/// CR 901.9 / 701.31 — the planar die rolled as an effect (Fractured
/// Powerstone) planeswalks on its Planeswalker face, and `Effect::Planeswalk`
/// turns the next plane up without a roll.
#[test]
fn cr_901_9_an_effect_rolls_the_planar_die_and_planeswalks() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, PlayerRef};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.seat_planar_deck(0, vec![catalog::naar_isle(), catalog::the_hippodrome()]);
    g.set_starting_plane(0);
    let plane = |g: &GameState| g.face_up_planes().first().and_then(|&id| g.find_card_anywhere(id)).map(|c| c.definition.name);
    assert_eq!(plane(&g), Some("Naar Isle"));
    let spell = |effect| CardDefinition { name: "Planar Test", card_types: vec![CardType::Sorcery], effect, ..Default::default() };
    let roll = g.add_card_to_hand(0, spell(Effect::RollPlanarDie { who: PlayerRef::You }));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1)]));
    g.perform_action(GameAction::CastSpell { card_id: roll, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(plane(&g), Some("The Hippodrome"), "the Planeswalker face planeswalked");
    let walk = g.add_card_to_hand(0, spell(Effect::Planeswalk { who: PlayerRef::You }));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: walk, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(plane(&g), Some("Naar Isle"));
}

/// CR 601.2c / 700.2 — a multi-mode cast validates each target against the
/// mode that owns it: choosing modes 1 and 2 of a "choose two" spell puts
/// mode 1's player target in slot 0, not under mode 0's spell filter.
#[test]
fn cr_700_2_each_chosen_mode_validates_its_own_target() {
    use crabomination::card::{CardDefinition, CardType, CounterType};
    use crabomination::effect::{Effect, Value};
    use crabomination::game::types::Target;
    use crabomination::effect::shortcut::target_filtered;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "Two-Mode Test",
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::CounterSpell { what: target_filtered(SelectionRequirement::IsSpellOnStack) },
                Effect::GainLife { who: target_filtered(SelectionRequirement::Player), amount: Value::Const(3) },
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ],
            min: 2,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    });
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpellSpree {
        card_id: spell,
        spree_modes: vec![1, 2],
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear)],
        x_value: None,
    })
    .expect("each target is checked against its own mode");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life + 3);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// CR 107.3 — a look-and-pick's "mana value X or less" reads the resolving
/// X, including one bound by `Effect::WithX` rather than a cast (Emergent
/// Woodwurm's "X is its power").
#[test]
fn cr_107_3_a_look_and_pick_filter_reads_the_resolving_x() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, LookPick, PlayerRef, Value};
    use crabomination::game::types::GameAction;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_library(0, catalog::serra_angel());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::plains());
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "X Dig Test",
        card_types: vec![CardType::Sorcery],
        effect: Effect::WithX {
            x: Value::Const(2),
            body: Box::new(Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(3),
                pick_filter: Some(SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMostXFromCost)),
                take: Some(Value::ONE),
                ..Default::default()
            }))),
        },
        ..Default::default()
    });
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "a 2-drop fits under X = 2");
}

/// CR 611.3a — a static over "creatures that attacked this turn" follows the
/// combat history live: the pump lands once the creature has attacked.
#[test]
fn cr_611_3a_a_static_over_attacked_this_turn_is_live() {
    use crabomination::card::{CardDefinition, CardType, StaticAbility, StaticEffect};
    use crabomination::effect::Selector;
    use crabomination::game::types::{Attack, AttackTarget, GameAction};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, CardDefinition {
        name: "Attack History Test",
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control that attacked this turn get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::AttackedThisTurn),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        ..Default::default()
    });
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

fn peace_main() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn run_as_spell(g: &mut GameState, effect: crabomination::effect::Effect) {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::game::types::GameAction;
    let id = g.add_card_to_hand(0, CardDefinition {
        name: "Primitive Test",
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    });
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn plain_token(name: &str, p: i32, t: i32) -> crabomination::card::TokenDefinition {
    use crabomination::card::{CardType, TokenDefinition};
    TokenDefinition { name: name.into(), power: p, toughness: t, card_types: vec![CardType::Creature], ..Default::default() }
}

/// CR 614.1a / 614.5 — "if you would create a Fish, create a Shark instead"
/// chains into "if you would create a Shark, create an Octopus instead",
/// each replacement applying once (Fisher's Talent at level 3).
#[test]
fn cr_614_5_named_token_replacements_chain_once_each() {
    use crabomination::card::{CardDefinition, CardType, StaticAbility, StaticEffect};
    use crabomination::effect::{Effect, PlayerRef, Value};
    let mut g = peace_main();
    let rule = |from: &str, into| StaticAbility {
        description: "token replacement",
        effect: StaticEffect::TokenNamedBecomes { name: from.into(), into },
    };
    g.add_card_to_battlefield(0, CardDefinition {
        name: "Token Chain Test",
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![rule("Fish", plain_token("Shark", 3, 3)), rule("Shark", plain_token("Octopus", 8, 8))],
        ..Default::default()
    });
    run_as_spell(&mut g, Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: std::sync::Arc::new(plain_token("Fish", 1, 1)),
    });
    let names: Vec<&str> = g.battlefield.iter().filter(|c| c.is_token).map(|c| c.definition.name).collect();
    assert_eq!(names, vec!["Octopus"]);
}

/// CR 105.4 — "choose a color other than green" never names green.
#[test]
fn cr_105_4_a_color_other_than_x_excludes_x() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::effect::Effect;
    use crabomination::game::types::GameAction;
    use crabomination::mana::Color;
    let mut g = peace_main();
    let land = g.add_card_to_hand(0, CardDefinition {
        name: "Other Color Test",
        card_types: vec![CardType::Land],
        as_enters_effect: Some(Effect::ChooseColorForSelfOtherThan(Color::Green)),
        ..Default::default()
    });
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    let chosen = g.battlefield_find(land).and_then(|c| c.chosen_color);
    assert!(chosen.is_some_and(|c| c != Color::Green), "chose {chosen:?}");
}

/// Kwain's offer: each player may draw; only the takers gain the life.
#[test]
fn cr_101_4_each_player_may_draw_and_takers_gain_life() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::Effect;
    let mut g = peace_main();
    for p in 0..2 {
        g.add_card_to_library(p, catalog::plains());
    }
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(false)]));
    run_as_spell(&mut g, Effect::EachPlayerMayDrawThenTakersGainLife { life: 1 });
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (1, l0 + 1));
    assert_eq!((g.players[1].hand.len(), g.players[1].life), (0, l1));
}

/// CR 114.4 — an emblem's "cast spells from your hand without paying their
/// mana costs" works from the command zone.
#[test]
fn cr_114_4_an_emblem_grants_free_casting_from_hand() {
    use crabomination::card::{StaticAbility, StaticEffect};
    use crabomination::effect::{Effect, PlayerRef};
    use crabomination::game::types::GameAction;
    let mut g = peace_main();
    run_as_spell(&mut g, Effect::CreateEmblem {
        who: PlayerRef::You,
        name: "Free Cast Test".into(),
        triggered: vec![],
        statics: vec![StaticAbility { description: "free", effect: StaticEffect::CastHandSpellsFree }],
    });
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: angel,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("no mana needed");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_some());
}

/// CR 603.3d — a "whenever you cast a spell" trigger with two target slots
/// binds both as it goes on the stack.
#[test]
fn cr_603_3d_a_cast_trigger_fills_every_target_slot() {
    use crabomination::card::{
        CardDefinition, CardType, CounterType, EventKind, EventScope, EventSpec, TriggeredAbility,
    };
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = peace_main();
    g.add_card_to_library(1, catalog::plains());
    g.add_card_to_battlefield(0, CardDefinition {
        name: "Two Slot Cast Test",
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::OpponentPlayer },
                    amount: Value::ONE,
                },
                Effect::AddCounter {
                    what: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Creature },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    });
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    run_as_spell(&mut g, Effect::Noop);
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(g.battlefield_find(angel).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}
