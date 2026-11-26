//! EDHREC singles short one card (`decks::cmdr_nov26`, `decks::cmdr_nov26_b`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn cast(g: &mut GameState, id: CardId) {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// Attack `defender` with `attacker`, no blocks, through combat damage.
fn hit(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    for _ in 0..64 {
        if matches!(g.step, TurnStep::EndCombat | TurnStep::PostCombatMain) {
            drain_stack(g);
            return;
        }
        let r = g.perform_action(GameAction::PassPriority);
        if r.is_err() && g.step == TurnStep::DeclareBlockers {
            g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
        }
    }
    panic!("combat never ended: step {:?}, stack {}, pending {:?}", g.step, g.stack.len(), g.pending_decision.is_some());
}

/// Massimo: a creature entering exiles an MV-1 instant from your graveyard and
/// gains "copy it on combat damage to a player"; the copy is cast free. An MV-2
/// instant is not a legal target, so nothing is granted.
#[test]
fn massimo_links_an_exiled_cantrip_to_the_entering_creature() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::massimo_the_magician());
    let opt = g.add_card_to_graveyard(0, catalog::opt());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != opt), "Opt exiled");
    assert_eq!(g.exile.iter().find(|c| c.id == opt).and_then(|c| c.exiled_with), Some(bear), "linked to the Bears");
    g.clear_sickness(bear);
    let (hand, life) = (g.players[0].hand.len(), g.players[2].life);
    hit(&mut g, bear, 2);
    assert_eq!(g.players[2].life, life - 2);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the free Opt copy drew a card");
    assert!(g.exile.iter().any(|c| c.id == opt), "the original stays exiled");

    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::massimo_the_magician());
    let cn = g.add_card_to_graveyard(0, catalog::counterspell());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cn), "MV 2: not a target");
    assert!(g.battlefield_find(bear).is_some_and(|c| c.definition.triggered_abilities.is_empty()));
}

// ── decks::cmdr_nov26_b ──────────────────────────────────────────────────────


/// `seat` resolves "`who` loses `n` life" as its own ability, then dispatches.
fn lose(g: &mut GameState, seat: usize, who: PlayerRef, n: i32) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    let evs = g
        .resolve_effect(&Effect::LoseLife { who: Selector::Player(who), amount: Value::Const(n) }, &ctx)
        .expect("lose life");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn plus_ones(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// Sahir: its own eminence ping (1 damage from a source you control) and a
/// 1-life loss your ability causes each grow it; the second resolution this
/// turn draws. A 2-life loss and an opponent's 1-life drain don't count, and
/// the ping is one trigger, not two (its life loss is the damage's).
#[test]
fn sahir_grows_on_exactly_one_from_your_own_sources() {
    let mut g = pod(3);
    let sahir = g.add_card_to_battlefield(0, catalog::sahir_visitor_in_darkness());
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "eminence ping");
    assert_eq!(plus_ones(&g, sahir), 1, "one trigger for one ping");
    assert_eq!(g.players[0].hand.len(), hand);
    lose(&mut g, 0, PlayerRef::You, 1);
    assert_eq!(plus_ones(&g, sahir), 2);
    assert_eq!(g.players[0].hand.len(), hand + 1, "second resolution this turn draws");
    lose(&mut g, 0, PlayerRef::You, 2);
    lose(&mut g, 1, PlayerRef::EachOpponent, 1);
    assert_eq!(plus_ones(&g, sahir), 2, "not exactly 1, or not your ability");
}

/// CR 113.6b — Sahir's eminence pings from the command zone, and the damage is
/// attributed to its owner (a source they control).
#[test]
fn cr_113_6b_sahir_pings_from_the_command_zone_as_its_owners_source() {
    let mut g = pod(2);
    g.seat_commanders(0, vec![catalog::sahir_visitor_in_darkness()]);
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1);
    let id = g.players[0].command[0].id;
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 1, Some(id), &mut ev);
    assert!(ev.iter().any(|e| matches!(e, GameEvent::DamageDealt { from_controller: Some(0), .. })));
}

/// Peace in Our Time: the active player may draw at their beginning of combat;
/// if they do, their creatures can't attack and a peace counter lands. Twelve
/// counters at its controller's upkeep win the game (CR 603.4 intervening if).
#[test]
fn peace_in_our_time_trades_attacks_for_cards_and_wins_at_twelve() {
    let mut g = pod(3);
    let peace = g.add_card_to_battlefield(1, catalog::peace_in_our_time());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the active player drew");
    assert_eq!(g.battlefield_find(peace).unwrap().counter_count(CounterType::Peace), 1);
    g.step = TurnStep::DeclareAttackers;
    let atk = GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }]);
    assert!(g.perform_action(atk).is_err(), "can't attack this turn");

    g.battlefield_find_mut(peace).unwrap().add_counters(CounterType::Peace, 11);
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].eliminated && g.players[2].eliminated, "twelve counters win");
}

/// Creeping Dread: everyone discards; only an opponent whose discard shares a
/// card type with yours loses 3.
#[test]
fn creeping_dread_punishes_matching_discards() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::creeping_dread());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::llanowar_elves());
    g.add_card_to_hand(2, catalog::island());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players.iter().all(|p| p.hand.is_empty()), "each player discarded");
    assert_eq!(g.players[1].life, l1 - 3, "creature matches creature");
    assert_eq!(g.players[2].life, l2, "a land doesn't match");
}

/// Dominion Infiltration: X=2 copies two opponents' creatures as hasty
/// changeling tokens for the caster, exiled at the next end step.
#[test]
fn dominion_infiltration_borrows_x_hasty_changeling_copies() {
    let mut g = pod(3);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::llanowar_elves());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::dominion_infiltration());
    g.players[0].mana_pool.add(Color::Red, 5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    let tokens: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).map(|c| c.id).collect();
    assert_eq!(tokens.len(), 2);
    for t in &tokens {
        let cp = g.computed_permanent(*t).unwrap();
        assert!(cp.keywords().contains(&Keyword::Haste) && cp.keywords().contains(&Keyword::Changeling));
    }
    assert!(g.battlefield_find(mine).is_some());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(tokens.iter().all(|t| g.battlefield_find(*t).is_none()), "exiled at the end step");
}

/// Bot: Peace in Our Time's draw is taken by its controller (the counter is
/// their win) and by a seat with nothing ready to attack, and declined by a
/// seat that has an attacker.
#[test]
fn bot_stands_down_for_peace_only_without_an_attack() {
    use crabomination::decision::OptionalKind;
    use crabomination::server::bot::decide_optional_trigger;
    let w = crabomination::server::EvalWeights::default();
    let desc = "Draw a card? (You can't attack this turn.)";
    let mut g = pod(3);
    let peace = g.add_card_to_battlefield(1, catalog::peace_in_our_time());
    let ask = |g: &GameState, seat| decide_optional_trigger(g, seat, peace, &OptionalKind::MayBody, desc, &w, false);
    assert!(ask(&g, 1), "the controller draws");
    assert!(ask(&g, 0), "nothing to attack with");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(ask(&g, 0), "a summoning-sick Bear can't attack");
    g.clear_sickness(bear);
    assert!(!ask(&g, 0), "an attacker ready: decline");
}
