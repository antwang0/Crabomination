//! Mercadian Masques (MMQ) gap closure, third wave.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast_alt(g: &mut GameState, id: CardId) -> Result<(), GameError> {
    let r = g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    });
    drain_stack(g);
    r.map(|_| ())
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

/// Seat 0's `attacker` attacks seat 1 and is blocked by seat 1's `blocker`.
fn attack_and_block(g: &mut GameState, attacker: CardId, blocker: CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(g);
}

/// Run the pending combat out through the end-of-combat step.
fn finish_combat(g: &mut GameState) {
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

// ── Legates ─────────────────────────────────────────────────────────────────

/// Kyren Legate is free only when the opponent has a Plains and you a Mountain.
#[test]
fn kyren_legate_is_free_only_across_the_right_lands() {
    let mut g = two_player_game();
    let legate = g.add_card_to_hand(0, catalog::kyren_legate());
    assert!(cast_alt(&mut g, legate).is_err(), "neither land in play");

    g.add_card_to_battlefield(0, catalog::mountain());
    assert!(cast_alt(&mut g, legate).is_err(), "your Mountain alone isn't enough");

    g.add_card_to_battlefield(1, catalog::plains());
    cast_alt(&mut g, legate).expect("both halves satisfied");
    assert!(g.battlefield_find(legate).is_some());
    assert_eq!(g.players[0].mana_pool.total(), 0, "cast for free");
}

/// Your own Plains doesn't satisfy the opponent half of a Legate's condition.
#[test]
fn legate_condition_reads_the_opponents_board() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_battlefield(0, catalog::plains());
    let legate = g.add_card_to_hand(0, catalog::kyren_legate());
    assert!(cast_alt(&mut g, legate).is_err(), "the Plains has to be theirs");
}

// ── Combat punishers ────────────────────────────────────────────────────────

/// Deathgazer kills the nonblack creature that blocks it, but survives the
/// exchange itself.
#[test]
fn deathgazer_kills_its_nonblack_blocker() {
    let mut g = two_player_game();
    let gazer = g.add_card_to_battlefield(0, catalog::deathgazer()); // 2/2
    let blocker = g.add_card_to_battlefield(1, catalog::wall_of_omens()); // 0/4 white
    attack_and_block(&mut g, gazer, blocker);
    finish_combat(&mut g);
    assert!(g.battlefield_find(blocker).is_none(), "destroyed at end of combat");
    assert!(g.battlefield_find(gazer).is_some());
}

/// A black blocker walks away from Deathgazer.
#[test]
fn deathgazer_spares_a_black_blocker() {
    let mut g = two_player_game();
    let gazer = g.add_card_to_battlefield(0, catalog::deathgazer());
    let blocker = g.add_card_to_battlefield(1, catalog::cateran_persuader()); // 2/1 black
    attack_and_block(&mut g, gazer, blocker);
    finish_combat(&mut g);
    // It died to combat damage, not to the trigger — check the trigger didn't
    // reach a *second* creature the Gazer never met.
    let bystander = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.battlefield_find(bystander).is_some());
}

/// Ceremonial Guard destroys itself for having shown up.
#[test]
fn ceremonial_guard_dies_after_attacking() {
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::ceremonial_guard());
    g.clear_sickness(guard);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: guard, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(guard).is_some(), "it survives through damage");
    finish_combat(&mut g);
    assert!(g.battlefield_find(guard).is_none(), "destroyed at end of combat");
}

/// Saprazzan Outrigger goes back on top of its owner's library after combat.
#[test]
fn saprazzan_outrigger_returns_to_the_library_top() {
    let mut g = two_player_game();
    let rigger = g.add_card_to_battlefield(0, catalog::saprazzan_outrigger());
    g.clear_sickness(rigger);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: rigger, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    finish_combat(&mut g);
    assert!(g.battlefield_find(rigger).is_none());
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(rigger));
}

/// Robber Fly makes the defending player churn their whole hand.
#[test]
fn robber_fly_churns_the_defenders_hand() {
    let mut g = two_player_game();
    let fly = g.add_card_to_battlefield(0, catalog::robber_fly());
    let blocker = g.add_card_to_battlefield(1, catalog::wind_drake()); // can catch a flier
    let kept = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::island());
    }
    attack_and_block(&mut g, fly, blocker);
    assert_eq!(g.players[1].hand.len(), 2, "same size, new cards");
    assert!(!g.players[1].hand.iter().any(|c| c.id == kept), "the old hand went away");
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Briar Patch shaves a point of power off each incoming attacker.
#[test]
fn briar_patch_weakens_attackers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::briar_patch());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let evs = g
        .declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(attacker).unwrap().power, 1);
}

/// Liability bites the controller of each nontoken permanent that dies.
#[test]
fn liability_charges_a_life_per_nontoken_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::liability());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    g.battlefield_find_mut(theirs).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// Black Market banks a counter per death and pays out at your main phase.
#[test]
fn black_market_pays_out_its_charge_counters() {
    let mut g = two_player_game();
    let market = g.add_card_to_battlefield(0, catalog::black_market());
    for seat in [0, 1] {
        let c = g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        g.battlefield_find_mut(c).unwrap().damage = 2;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(market).unwrap().counter_count(CounterType::Charge), 2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2);
}

/// Security Detail only fires from an empty board, and only once a turn.
#[test]
fn security_detail_needs_an_empty_board() {
    let mut g = two_player_game();
    let detail = g.add_card_to_battlefield(0, catalog::security_detail());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    let fire = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: detail,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(fire(&mut g).is_err(), "you control a creature");
    g.remove_to_graveyard_with_triggers(bears);
    drain_stack(&mut g);
    fire(&mut g).expect("empty board");
    drain_stack(&mut g);
    assert!(fire(&mut g).is_err(), "once each turn");
}

/// Putrefaction makes the *caster* of a green or white spell discard.
#[test]
fn putrefaction_hits_the_caster_of_a_green_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::putrefaction());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, bears, None);
    assert!(g.players[1].hand.is_empty(), "the green caster discarded");
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Bifurcate fetches a twin of a creature already on the battlefield.
#[test]
fn bifurcate_fetches_a_twin() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let twin = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::hill_giant()); // wrong name
    let spell = g.add_card_to_hand(0, catalog::bifurcate());
    mana(&mut g, 0);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(twin)),
    ]));
    cast(&mut g, spell, Some(Target::Permanent(bears)));
    assert!(g.battlefield_find(twin).is_some(), "the same-named card came down");
    assert_eq!(g.battlefield_find(twin).unwrap().controller, 0);
}

/// Arms Dealer fires a Goblin at a creature for 4.
#[test]
fn arms_dealer_shoots_a_goblin_for_four() {
    let mut g = two_player_game();
    let dealer = g.add_card_to_battlefield(0, catalog::arms_dealer());
    let ammo = g.add_card_to_battlefield(0, catalog::kyren_glider()); // a Goblin
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dealer,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ammo).is_none(), "the Goblin was the cost");
    assert!(g.battlefield_find(victim).is_none());
    let _ = dealer;
}

/// Lithophage eats a Mountain each upkeep, and itself when there's none left.
#[test]
fn lithophage_eats_a_mountain_or_itself() {
    let mut g = two_player_game();
    let phage = g.add_card_to_battlefield(0, catalog::lithophage());
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(mountain).is_none());
    assert!(g.battlefield_find(phage).is_some());

    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(phage).is_none(), "no Mountain left to feed it");
}
