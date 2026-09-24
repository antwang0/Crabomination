//! Commander: the Open Hostility precon (C16, Saskia, `decks::cmdr_saskia`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_x(g, id, targets, None);
}

/// Attack `defender` with `attackers`, no blocks, through combat damage.
fn swing(g: &mut GameState, attackers: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let atks = attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(defender) }).collect();
    g.perform_action(GameAction::DeclareAttackers(atks)).expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = defender;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Life lost (negative: gained) since the game began.
fn lost(g: &GameState, s: usize) -> i32 {
    g.players[s].starting_life - g.players[s].life
}

/// Saskia: a creature's combat damage to one player is dealt again, by that
/// creature, to the chosen player.
#[test]
fn saskia_echoes_combat_damage_to_the_chosen_player() {
    let mut g = pod(3);
    let saskia = g.add_card_to_hand(0, catalog::saskia_the_unyielding());
    cast(&mut g, saskia, &[]);
    let chosen = g.battlefield_find(saskia).unwrap().chosen_player.expect("a player was chosen");
    assert_ne!(chosen, 0);
    let other = if chosen == 1 { 2 } else { 1 };
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.clear_sickness(giant);
    swing(&mut g, &[giant], other);
    assert_eq!(lost(&g, other), 3);
    assert_eq!(lost(&g, chosen), 3, "Saskia sends the 3 on");
}

/// CR 702.80a — Everlasting Torment: a burn spell's damage to a creature
/// lands as -1/-1 counters, and nobody gains life.
#[test]
fn everlasting_torment_withers_everything() {
    let mut g = pod(2);
    let et = g.add_card_to_hand(0, catalog::everlasting_torment());
    cast(&mut g, et, &[]);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(wurm)]);
    let w = g.battlefield_find(wurm).unwrap();
    assert_eq!(w.damage, 0);
    assert_eq!(w.counter_count(CounterType::MinusOneMinusOne), 3);
    let before = lost(&g, 0);
    let heal = g.add_card_to_hand(0, catalog::healing_salve());
    cast(&mut g, heal, &[Target::Player(0)]);
    assert_eq!(lost(&g, 0), before);
}

/// Conqueror's Flail: once equipped, opponents can't cast during your turn;
/// unattached, it doesn't lock.
#[test]
fn conquerors_flail_locks_only_while_attached() {
    let mut g = pod(2);
    let flail = g.add_card_to_battlefield(0, catalog::conquerors_flail());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let try_bolt = |g: &mut GameState| {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        flood(g, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(try_bolt(&mut g).is_ok(), "unattached: no lock");
    drain_stack(&mut g);
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: flail, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(flail).unwrap().attached_to, Some(bear));
    // Grizzly Bears is green; Flail is colorless: one color.
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(try_bolt(&mut g).is_err(), "attached: opponents are silenced on your turn");
}

/// Charging Cinderhorn: an end step with no attackers adds a fury counter and
/// burns the active player for the count.
#[test]
fn charging_cinderhorn_punishes_peace() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::charging_cinderhorn());
    for s in 0..2 {
        for _ in 0..3 {
            g.add_card_to_library(s, catalog::island());
        }
    }
    g.active_player_idx = 1;
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 1;
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(lost(&g, 1), 1);
}

/// Tymna: after combat damage to an opponent, pay 1 life to draw 1.
#[test]
fn tymna_draws_per_opponent_hit() {
    let mut g = pod(3);
    let tymna = g.add_card_to_battlefield(0, catalog::tymna_the_weaver());
    g.clear_sickness(tymna);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    swing(&mut g, &[tymna], 1);
    drain_stack(&mut g);
    // Lifelink +2, then pay 1 for the one opponent hit.
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(lost(&g, 0), -2 + 1);
}

/// Lavalanche hits the player and every creature they control, not yours.
#[test]
fn lavalanche_sweeps_one_player() {
    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lv = g.add_card_to_hand(0, catalog::lavalanche());
    cast_x(&mut g, lv, &[Target::Player(1)], Some(2));
    assert_eq!(lost(&g, 1), 2);
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(mine).is_some());
}

/// Divergent Transformations: each exiled creature's controller reveals to
/// a creature card and puts it onto the battlefield.
#[test]
fn divergent_transformations_rerolls_two_creatures() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::craw_wurm());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::hill_giant());
    let dt = g.add_card_to_hand(0, catalog::divergent_transformations());
    cast(&mut g, dt, &[Target::Permanent(a), Target::Permanent(b)]);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    let named = |s: usize, n: &str| g.battlefield.iter().any(|c| c.controller == s && c.definition.name == n);
    assert!(named(0, "Craw Wurm"));
    assert!(named(1, "Hill Giant"));
}

/// Wilderness Elemental counts the nonbasic lands opponents control.
#[test]
fn wilderness_elemental_counts_opposing_nonbasics() {
    let mut g = pod(2);
    let we = g.add_card_to_battlefield(0, catalog::wilderness_elemental());
    g.add_card_to_battlefield(1, catalog::command_tower());
    g.add_card_to_battlefield(1, catalog::island());
    g.add_card_to_battlefield(0, catalog::command_tower());
    let cp = g.computed_permanent(we).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 3));
}

/// Brutal Hordechief: each attacker drains its defending player for 1.
#[test]
fn brutal_hordechief_drains_per_attacker() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::brutal_hordechief());
    let bears: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    for &b in &bears {
        g.clear_sickness(b);
    }
    swing(&mut g, &bears, 1);
    assert_eq!(lost(&g, 1), 2 + 4);
    assert_eq!(lost(&g, 0), -2);
}

/// Primeval Protector costs {1} less per opposing creature and grows your
/// other creatures.
#[test]
fn primeval_protector_discount_and_counters() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pp = g.add_card_to_hand(0, catalog::primeval_protector());
    g.players[0].mana_pool.add(Color::Green, 7);
    g.perform_action(GameAction::CastSpell {
        card_id: pp,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("seven mana with four opposing creatures");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(pp).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Stonehoof Chieftain grants trample and indestructible to other attackers.
#[test]
fn stonehoof_chieftain_hardens_attackers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::stonehoof_chieftain());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Indestructible));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Trample));
}
