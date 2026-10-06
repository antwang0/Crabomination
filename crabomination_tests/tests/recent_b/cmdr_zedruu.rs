//! Commander: the Political Puppets precon (CMD, Zedruu, `decks::cmdr_zedruu`).

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

fn cast_by(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], drain: bool) {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    if drain {
        drain_stack(g);
    }
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_by(g, 0, id, targets, true);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn step_into(g: &mut GameState, active: usize, from: TurnStep, to: TurnStep) {
    g.active_player_idx = active;
    g.step = from;
    g.priority.player_with_priority = active;
    while g.step != to {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

/// CR 702.24 — Jötun Grunt's cumulative upkeep bottoms two cards of a single
/// graveyard per age counter (an opponent's first); with too few, it's
/// sacrificed.
#[test]
fn jotun_grunt_eats_graveyards_or_dies() {
    let mut g = pod(2);
    let grunt = g.add_card_to_battlefield(0, catalog::jotun_grunt());
    for _ in 0..3 {
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
    }
    step_into(&mut g, 0, TurnStep::Untap, TurnStep::Draw);
    assert!(g.battlefield_find(grunt).is_some());
    assert_eq!(g.players[1].graveyard.len(), 1);
    assert_eq!(g.players[1].library.len(), 2);
    step_into(&mut g, 0, TurnStep::Untap, TurnStep::Draw);
    assert!(g.battlefield_find(grunt).is_none(), "two age counters want four cards");
}

/// CR 702.24 + Jötun Grunt's 2006-07-15 ruling — a deciding seat picks the
/// graveyard and the cards for each age counter, and each counter may use a
/// different graveyard.
#[test]
fn jotun_grunt_seat_picks_graveyard_and_cards_per_counter() {
    let mut g = pod(3);
    let grunt = g.add_card_to_battlefield(0, catalog::jotun_grunt());
    g.battlefield_find_mut(grunt).unwrap().add_counters(CounterType::Age, 1);
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(1, catalog::forest());
    let bears = g.add_card_to_graveyard(2, catalog::grizzly_bears());
    let ring = g.add_card_to_graveyard(2, catalog::sol_ring());
    let land = g.add_card_to_graveyard(2, catalog::forest());
    let mine: Vec<CardId> = g.players[0].graveyard.iter().map(|c| c.id).collect();
    g.players[0].wants_ui = true;
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    let _ = g.advance_step(Vec::new());
    while g.pending_decision.is_none() && !g.stack.is_empty() {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    for answer in [
        DecisionAnswer::Bool(true),
        DecisionAnswer::Amount(1), // [seat 0, seat 2] → seat 2
        DecisionAnswer::Cards(vec![ring, land]),
        DecisionAnswer::Amount(0), // seat 2 has one left → only seat 0
        DecisionAnswer::Cards(mine),
    ] {
        assert!(g.pending_decision.is_some(), "asked before {answer:?}");
        g.submit_decision(answer).expect("answer");
    }
    assert!(g.pending_decision.is_none());
    drain_stack(&mut g);
    assert!(g.battlefield_find(grunt).is_some());
    assert!(g.players[0].graveyard.is_empty());
    assert_eq!(g.players[1].graveyard.len(), 1);
    let left: Vec<CardId> = g.players[2].graveyard.iter().map(|c| c.id).collect();
    assert_eq!(left, vec![bears], "the seat's pick, not the highest mana values");
}

/// Martyr's Bond: your creature dies → each opponent sacrifices a creature;
/// the Bond itself going → each opponent sacrifices an enchantment.
#[test]
fn martyrs_bond_makes_the_table_match_your_losses() {
    let mut g = pod(3);
    let bond = g.add_card_to_battlefield(0, catalog::martyrs_bond());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for s in [1, 2] {
        g.add_card_to_battlefield(s, catalog::grizzly_bears());
        g.add_card_to_battlefield(s, catalog::sol_ring());
        g.add_card_to_battlefield(s, catalog::crescendo_of_war());
    }
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[Target::Permanent(mine)]);
    for s in [1, 2] {
        assert!(!g.battlefield.iter().any(|c| c.controller == s && c.definition.is_creature()), "seat {s}");
        assert!(g.battlefield.iter().any(|c| c.controller == s && c.definition.name == "Sol Ring"));
    }
    let ench = g.add_card_to_hand(0, catalog::disenchant());
    cast(&mut g, ench, &[Target::Permanent(bond)]);
    for s in [1, 2] {
        assert!(!g.battlefield.iter().any(|c| c.controller == s && c.definition.name == "Crescendo of War"));
    }
}

/// Spell Crumple: the countered spell and Spell Crumple both go to the
/// bottom of their owners' libraries.
#[test]
fn spell_crumple_bottoms_both() {
    let mut g = pod(2);
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_by(&mut g, 1, bolt, &[Target::Player(0)], false);
    let crumple = g.add_card_to_hand(0, catalog::spell_crumple());
    cast_by(&mut g, 0, crumple, &[Target::Permanent(bolt)], true);
    assert_eq!(g.players[0].life, g.players[0].starting_life);
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(bolt));
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(crumple));
}

/// CR 701.30 — Pollen Lullaby's clash, won, keeps *the clashed opponent's*
/// creatures tapped through their next untap.
#[test]
fn pollen_lullaby_locks_the_clashed_opponent() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::craw_wurm());
    g.add_card_to_library(1, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let pl = g.add_card_to_hand(0, catalog::pollen_lullaby());
    cast(&mut g, pl, &[]);
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Whirlpool Whelm: a won clash lets its caster put the creature on top of its
/// owner's library ("you may"); declined, or a lost clash, bounces it to hand.
#[test]
fn whirlpool_whelm_tops_on_a_win() {
    let wins = || (catalog::craw_wurm(), catalog::island());
    for ((mine, theirs), take, on_top) in
        [(wins(), true, true), (wins(), false, false), ((catalog::island(), catalog::craw_wurm()), true, false)]
    {
        let mut g = pod(2);
        // Both clashers keep their card on top, then the caster's "you may".
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(false),
            DecisionAnswer::Bool(false),
            DecisionAnswer::Bool(take),
        ]));
        g.add_card_to_library(0, mine);
        g.add_card_to_library(1, theirs);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ww = g.add_card_to_hand(0, catalog::whirlpool_whelm());
        cast(&mut g, ww, &[Target::Permanent(bear)]);
        assert!(g.battlefield_find(bear).is_none());
        assert_eq!(g.players[1].library.first().map(|c| c.id) == Some(bear), on_top);
        assert_eq!(g.players[1].hand.iter().any(|c| c.id == bear), !on_top);
    }
}

/// Brion Stoutarm flings another creature for its power at a player.
#[test]
fn brion_stoutarm_flings() {
    let mut g = pod(2);
    let brion = g.add_card_to_battlefield(0, catalog::brion_stoutarm());
    g.clear_sickness(brion);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    activate(&mut g, brion, Some(Target::Player(1)), None);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[1].life, g.players[1].starting_life - 3);
}

/// Crescendo of War: a strife counter each upkeep; attackers get +1/+0 per
/// counter.
#[test]
fn crescendo_of_war_escalates() {
    let mut g = pod(2);
    let cw = g.add_card_to_battlefield(0, catalog::crescendo_of_war());
    for s in 0..2 {
        g.add_card_to_library(s, catalog::island());
    }
    step_into(&mut g, 1, TurnStep::Untap, TurnStep::Draw);
    step_into(&mut g, 0, TurnStep::Untap, TurnStep::Draw);
    assert_eq!(g.battlefield_find(cw).unwrap().counter_count(CounterType::Strife), 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, bear), (2, 2));
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    assert_eq!(pt(&g, bear), (4, 2));
}

/// Nin: X damage to a creature, and its controller draws X.
#[test]
fn nin_pays_the_victim() {
    let mut g = pod(2);
    let nin = g.add_card_to_battlefield(0, catalog::nin_the_pain_artist());
    g.clear_sickness(nin);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    activate(&mut g, nin, Some(Target::Permanent(wurm)), Some(2));
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 2);
    assert_eq!(g.players[1].hand.len(), 2);
}

/// Prison Term hops to an opponent's creature as it enters.
#[test]
fn prison_term_jumps_to_the_newcomer() {
    let mut g = pod(2);
    let first = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pt_id = g.add_card_to_hand(0, catalog::prison_term());
    cast(&mut g, pt_id, &[Target::Permanent(first)]);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let big = g.add_card_to_hand(1, catalog::craw_wurm());
    g.active_player_idx = 1;
    cast_by(&mut g, 1, big, &[], true);
    assert_eq!(g.battlefield_find(pt_id).unwrap().attached_to, Some(big));
}

/// CR 508.1d — Ruhan must attack the opponent it picked at random this
/// combat, and only this combat.
#[test]
fn ruhan_attacks_its_random_opponent() {
    let mut g = pod(3);
    let ruhan = g.add_card_to_battlefield(0, catalog::ruhan_of_the_fomori());
    g.clear_sickness(ruhan);
    step_into(&mut g, 0, TurnStep::PreCombatMain, TurnStep::BeginCombat);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let at = |p| GameAction::DeclareAttackers(vec![Attack { attacker: ruhan, target: AttackTarget::Player(p) }]);
    let legal: Vec<usize> = [1, 2].into_iter().filter(|&q| g.clone().perform_action(at(q)).is_ok()).collect();
    assert_eq!(legal.len(), 1, "only the picked opponent: {legal:?}");
    assert!(g.clone().perform_action(GameAction::DeclareAttackers(vec![])).is_err(), "it must attack");
    step_into(&mut g, 0, TurnStep::EndCombat, TurnStep::PostCombatMain);
    g.step = TurnStep::DeclareAttackers;
    let free = [1, 2].into_iter().filter(|&q| g.clone().perform_action(at(q)).is_ok()).count();
    assert_eq!(free, 2, "the requirement ends with the combat");
}

/// CR 508.1d — Ruhan's pick is its own requirement: a second one naming the
/// other opponent (the Fantastic Four's forced attack) leaves a tie, so the
/// controller picks either, where the later stamp used to overwrite Ruhan's.
#[test]
fn ruhan_and_a_second_forced_attack_tie() {
    use crabomination::effect::{Effect, PlayerRef, Selector};
    let mut g = pod(3);
    let ruhan = g.add_card_to_battlefield(0, catalog::ruhan_of_the_fomori());
    g.clear_sickness(ruhan);
    step_into(&mut g, 0, TurnStep::PreCombatMain, TurnStep::BeginCombat);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let at = |p| GameAction::DeclareAttackers(vec![Attack { attacker: ruhan, target: AttackTarget::Player(p) }]);
    let pick = [1, 2].into_iter().find(|&q| g.clone().perform_action(at(q)).is_ok()).expect("a pick");
    let other = 3 - pick;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::MustAttackPlayerThisTurn {
            attacker: Selector::ExactObjects(vec![ruhan]),
            defender: Selector::Player(PlayerRef::Seat(other)),
        },
        &ctx,
    )
    .expect("forced attack");
    for q in [pick, other] {
        g.clone().perform_action(at(q)).unwrap_or_else(|e| panic!("seat {q} obeys one of the two: {e:?}"));
    }
}

/// Rapacious One: combat damage to a player makes that many Eldrazi Spawn.
#[test]
fn rapacious_one_spawns_per_damage() {
    let mut g = pod(2);
    let r1 = g.add_card_to_battlefield(0, catalog::rapacious_one());
    g.clear_sickness(r1);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: r1, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Eldrazi Spawn").count(), 5);
}
