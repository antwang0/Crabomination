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
                single_player_value_misuse(inner, out);
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(|i| single_player_value_misuse(i, out)),
        _ => {}
    }
}

#[test]
fn cr_800_4_no_single_player_value_reads_each_opponent() {
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
