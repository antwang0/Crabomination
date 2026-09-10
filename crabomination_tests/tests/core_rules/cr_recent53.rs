//! CR conformance for this run's engine work:
//! - CR 113.3 — any-player activated abilities (Damping Engine).
//! - CR 120.4a — excess damage redirected before the damage event.
//! - CR 208.3a — a P/T-modifying effect on a noncreature permanent is created
//!   anyway and applies once that permanent becomes a creature.
//! - CR 611.2c — "for as long as this remains tapped" continuous effects.

use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Duration, Effect, Selector, Value};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::TurnStep;

fn activate(g: &mut GameState, id: CardId, idx: usize) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

// ── CR 113.3 — abilities ────────────────────────────────────────────────────

/// CR 113.3 — an ability flagged "any player may activate this" is usable by a
/// player who doesn't control the source; ordinary abilities are not.
#[test]
fn cr_113_3_any_player_may_activate_a_flagged_ability() {
    let mut g = two_player_game();
    let engine = g.add_card_to_battlefield(0, catalog::damping_engine());
    let jar = g.add_card_to_battlefield(0, catalog::memory_jar());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: jar,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "an ordinary ability stays with its controller"
    );
    activate(&mut g, engine, 0);
    assert!(g.players[1].statics_ignored_this_turn.contains(&engine));
}

// ── CR 120.4a — excess damage ───────────────────────────────────────────────

/// CR 120.4a — the redirect happens before the damage is dealt, so the creature
/// is marked for exactly lethal and only the rest lands on its controller.
#[test]
fn cr_120_4a_excess_is_split_off_before_the_damage_event() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(
        0,
        Some(Target::Permanent(bear)),
        0,
        0,
    );
    g.resolve_effect(
        &Effect::DealDamageExcessToController {
            to: Selector::Target(0),
            amount: Value::Const(6),
        },
        &ctx,
    )
    .expect("resolve");
    assert_eq!(g.players[1].life, life - 4, "6 minus the lethal 2 spilled over");
    assert!(g.battlefield_find(bear).is_none());
}

/// CR 120.4a — a deathtouch source makes everything past 1 damage excess.
#[test]
fn cr_120_4a_deathtouch_makes_one_damage_lethal() {
    let mut g = two_player_game();
    let rats = g.add_card_to_battlefield(0, catalog::typhoid_rats());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        rats,
        0,
        Some(Target::Permanent(bear)),
    );
    g.resolve_effect(
        &Effect::DealDamageExcessToController {
            to: Selector::Target(0),
            amount: Value::Const(6),
        },
        &ctx,
    )
    .expect("resolve");
    assert_eq!(g.players[1].life, life - 5, "only 1 was needed on the body");
}

// ── CR 208.3a — P/T effects on noncreature permanents ───────────────────────

/// CR 208.3a — a +N/+N effect created against a noncreature permanent exists
/// and starts applying the moment that permanent becomes a creature.
#[test]
fn cr_208_3a_pump_on_a_noncreature_applies_once_it_animates() {
    let mut g = two_player_game();
    let jar = g.add_card_to_battlefield(0, catalog::memory_jar());
    let ctx = crabomination::game::effects::EffectContext::for_spell(
        0,
        Some(Target::Permanent(jar)),
        0,
        0,
    );
    g.resolve_effect(
        &Effect::PumpPT {
            what: Selector::Target(0),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("pump");
    assert!(!g.computed_permanent(jar).unwrap().card_types().contains(&CardType::Creature));
    g.resolve_effect(
        &Effect::BecomeCreature {
            what: Selector::Target(0),
            power: Value::Const(3),
            toughness: Value::Const(3),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("animate");
    let cp = g.computed_permanent(jar).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "the banked pump is live now");
}

// ── CR 611.2c — "for as long as" durations ──────────────────────────────────

/// CR 611.2c — the affected set is locked in when the effect starts, and the
/// effect ends when its stated condition (the source staying tapped) fails.
#[test]
fn cr_611_2c_while_source_tapped_locks_its_set_and_expires_on_untap() {
    let mut g = two_player_game();
    let weaponry = g.add_card_to_battlefield(0, catalog::thran_weaponry());
    let early = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, weaponry, 0);
    let late = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(early).unwrap().power, 4);
    assert_eq!(g.computed_permanent(late).unwrap().power, 2, "joined after the lock-in");
    g.battlefield_find_mut(weaponry).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(early).unwrap().power, 2);
}

// ── CR 209 / 210 — loyalty and defense ──────────────────────────────────────

/// CR 209.1 / 306.5b — a planeswalker enters with loyalty counters equal to its
/// printed loyalty, and that number is its loyalty off the battlefield too.
#[test]
fn cr_209_1_planeswalker_enters_with_its_printed_loyalty() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let def = catalog::saheeli_sublime_artificer();
    let printed = def.base_loyalty;
    assert!(printed > 0, "the printed loyalty is the off-battlefield value");
    let pw = g.add_card_to_battlefield(0, def);
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty), printed);
}

/// CR 210.1 / 310.4b — a battle enters with defense counters equal to its
/// printed defense.
#[test]
fn cr_210_1_battle_enters_with_its_printed_defense() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let def = catalog::invasion_of_zendikar();
    let printed = def.defense;
    assert!(printed > 0);
    let battle = g.add_card_to_battlefield(0, def);
    assert_eq!(g.battlefield_find(battle).unwrap().counter_count(CounterType::Defense), printed);
}

// ── CR 308 — Kindred ────────────────────────────────────────────────────────

/// CR 308.1/308.2 — a kindred card has another card type and carries creature
/// types, so a type lord's anthem sees it as a Goblin without it being a
/// creature.
#[test]
fn cr_308_2_kindred_permanents_carry_creature_types_without_being_creatures() {
    use crabomination::card::{CardType, CreatureType};
    let mut g = two_player_game();
    let altar = g.add_card_to_battlefield(0, catalog::altar_of_the_goyf());
    let cp = g.computed_permanent(altar).unwrap();
    assert!(cp.card_types().contains(&CardType::Artifact));
    assert!(!cp.card_types().contains(&CardType::Creature), "a kindred artifact isn't a creature");
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Lhurgoyf), "but it has the type");
}

// ── CR 603.2c / 603.3d / 603.4 / 603.6 — the graveyard walk ──────────────────
// `dispatch_triggers_for_events` walks graveyards for `FromYourGraveyard`
// triggers under the battlefield walk's rules: fan-out by kind, "once each
// turn" capping a batch at one fire and spending its slot only when the
// intervening if holds, "one or more" capping a batch without a turn cap.

/// CR 603.6 — a graveyard-resident "whenever an opponent gains life" fires
/// once per life-gain event in a batch, as it would from the battlefield.
#[test]
fn cr_603_6_a_graveyard_trigger_fans_out_per_matching_event() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::punishing_fire());
    g.dispatch_triggers_for_events(&[
        GameEvent::LifeGained { player: 1, amount: 2 },
        GameEvent::LifeGained { player: 1, amount: 3 },
    ]);
    assert_eq!(g.stack.len(), 2, "one Punishing Fire trigger per life gain");
}

/// CR 603.3d — a graveyard-resident "once each turn" trigger fires once for a
/// batch of three draws and not again that turn.
#[test]
fn cr_603_3d_a_graveyard_once_per_turn_trigger_fires_once_a_batch_and_once_a_turn() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::sneaky_snacker());
    let c = g.add_card_to_library(0, catalog::forest());
    g.players[0].cards_drawn_this_turn = 3;
    let draw = GameEvent::CardDrawn { player: 0, card_id: c };
    g.dispatch_triggers_for_events(&[draw.clone(), draw.clone(), draw.clone()]);
    assert_eq!(g.stack.len(), 1, "a draw-three batch mints one trigger");
    g.dispatch_triggers_for_events(&[draw]);
    assert_eq!(g.stack.len(), 1, "the slot is spent for the turn");
}

/// CR 603.4 — a failed intervening if does not spend the once-per-turn slot:
/// the second draw fails Sneaky Snacker's "your third card", the third fires.
#[test]
fn cr_603_4_a_failed_intervening_if_keeps_the_graveyard_once_per_turn_slot() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::sneaky_snacker());
    let c = g.add_card_to_library(0, catalog::forest());
    let draw = GameEvent::CardDrawn { player: 0, card_id: c };
    g.players[0].cards_drawn_this_turn = 2;
    g.dispatch_triggers_for_events(std::slice::from_ref(&draw));
    assert!(g.stack.is_empty(), "the second draw is not the third");
    g.players[0].cards_drawn_this_turn = 3;
    g.dispatch_triggers_for_events(&[draw]);
    assert_eq!(g.stack.len(), 1, "the third draw fires");
}

/// CR 603.2c — "whenever one or more cards leave your graveyard" fires once
/// for a batch of two and again for the next batch (no per-turn cap).
#[test]
fn cr_603_2c_once_per_batch_fires_once_a_batch_and_again_next_batch() {
    let mut g = two_player_game();
    let hunter = g.add_card_to_battlefield(0, catalog::attuned_hunter());
    let c = g.add_card_to_library(0, catalog::forest());
    let left = GameEvent::CardLeftGraveyard { player: 0, card_id: c };
    g.dispatch_triggers_for_events(&[left.clone(), left.clone()]);
    assert_eq!(g.stack.len(), 1, "two cards leaving at once mint one trigger");
    drain_stack(&mut g);
    g.dispatch_triggers_for_events(&[left]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hunter).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "a later batch fires again"
    );
}

/// CR 603.2c — "whenever one or more creatures you control with flying deal
/// combat damage to a player" (Mu Yanling, Wind Rider) fires once for two
/// flyers connecting in one damage batch, not once per dealer.
#[test]
fn cr_603_2c_one_or_more_combat_damage_fires_once_a_damage_batch() {
    let mut g = two_player_game();
    let mu = g.add_card_to_battlefield(0, catalog::mu_yanling_wind_rider());
    g.battlefield_find_mut(mu).unwrap().counters.insert(CounterType::Loyalty, 5);
    let a = g.add_card_to_battlefield(0, catalog::suntail_hawk());
    let b = g.add_card_to_battlefield(0, catalog::storm_crow());
    g.clear_sickness(a);
    g.clear_sickness(b);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand = g.players[0].hand.len();
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "both flyers connected");
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for the batch");
}

/// CR 603.2c — "whenever one or more other creatures you control die"
/// (Vengeful Townsfolk) fires once for a batch of two deaths and again for a
/// later death in the same turn: a batch cap, not a turn cap.
#[test]
fn cr_603_2c_one_or_more_deaths_fires_once_a_batch_and_again_later_in_the_turn() {
    let mut g = two_player_game();
    let vt = g.add_card_to_battlefield(0, catalog::vengeful_townsfolk());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[
        GameEvent::CreatureDied { card_id: a },
        GameEvent::CreatureDied { card_id: b },
    ]);
    assert_eq!(g.stack.len(), 1, "two deaths at once mint one trigger");
    drain_stack(&mut g);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: a }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(vt).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "a later death fires again"
    );
}

// ── CR 605.3a / 118.3 — paying a "you may pay" mid-resolution ─────────────

/// CR 605.3a — a "you may pay {R}" trigger resolving with an empty pool taps
/// the controller's Mountain for it (the echo path), instead of silently
/// failing the payment: Punishing Fire comes back to hand.
#[test]
fn cr_605_3a_a_you_may_pay_trigger_taps_a_land_when_the_pool_is_empty() {
    let mut g = two_player_game();
    let fire = g.add_card_to_graveyard(0, catalog::punishing_fire());
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    assert_eq!(g.players[0].mana_pool.total(), 0);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 1, amount: 2 }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == fire), "Punishing Fire returned");
    assert!(g.battlefield_find(mountain).unwrap().tapped, "the Mountain paid for it");
}

/// CR 605.3a — with no source to tap the payment fails and the body is
/// skipped, as before.
#[test]
fn cr_605_3a_an_unfundable_you_may_pay_runs_nothing() {
    let mut g = two_player_game();
    let fire = g.add_card_to_graveyard(0, catalog::punishing_fire());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 1, amount: 2 }]);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fire), "stays in the graveyard");
}
