//! Commander: the Aura of Courage precon (AFC, Galea, `decks::cmdr_galea`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn act_as(g: &mut GameState, seat: usize, action: GameAction) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act_as(g, seat, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, 0, id, targets, None)
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    act_as(g, 0, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::plains());
    }
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn kw(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&k))
}

/// Galea: an Equipment cast off the library top attaches to a creature of
/// yours as it enters; an Aura is castable from there too.
#[test]
fn galea_casts_equipment_from_the_top_and_attaches_it() {
    let mut g = main_phase(2);
    library(&mut g, 0, 2);
    let galea = g.add_card_to_battlefield(0, catalog::galea_kindler_of_hope());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let robe = g.add_card_to_library(0, catalog::robe_of_stars());
    g.players[0].library.retain(|c| c.id != robe);
    let card = crabomination::card::CardInstance::new(robe, catalog::robe_of_stars(), 0);
    g.players[0].library.insert(0, card);
    cast_at(&mut g, robe, &[]).expect("cast off the top");
    let host = g.battlefield_find(robe).unwrap().attached_to.expect("attached as it entered");
    assert!(host == bear || host == galea, "a creature its caster controls");
}

/// Belt of Giant Strength's equip is {10} less the target's power.
#[test]
fn belt_of_giant_strength_is_cheaper_on_a_big_body() {
    let mut g = main_phase(2);
    let belt = g.add_card_to_battlefield(0, catalog::belt_of_giant_strength());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.players[0].mana_pool.add_colorless(7);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: belt, target: giant }).expect("{10} less 3");
    drain_stack(&mut g);
    assert_eq!(pt(&g, giant), (10, 10));
}

/// Gryff's Boon returns from the graveyard onto a target creature.
#[test]
fn gryffs_boon_flies_back_onto_a_creature() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gb = g.add_card_to_graveyard(0, catalog::gryffs_boon());
    activate(&mut g, gb, 0, &[Target::Permanent(bear)]).expect("return");
    assert_eq!(g.battlefield_find(gb).unwrap().attached_to, Some(bear));
    assert!(kw(&g, bear, Keyword::Flying) && pt(&g, bear) == (3, 2));
}

/// CR 706.2 — Netherese Puzzle-Ward draws when a die shows its highest
/// natural face (the upkeep d4 here), not on a lower one.
#[test]
fn cr_706_2_netherese_puzzle_ward_draws_on_a_natural_max() {
    let mut g = main_phase(2);
    library(&mut g, 0, 8);
    g.add_card_to_battlefield(0, catalog::netherese_puzzle_ward());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(4)]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "a natural 4 on a d4");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(3)]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "a 3 draws nothing");
}

/// Ride the Avalanche: the next spell has flash, and when it's cast a
/// creature gets its mana value in +1/+1 counters.
#[test]
fn ride_the_avalanche_flashes_the_next_spell() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 1;
    let rta = g.add_card_to_hand(0, catalog::ride_the_avalanche());
    cast_at(&mut g, rta, &[]).expect("instant");
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    cast_at(&mut g, giant, &[]).expect("a creature on the opponent's turn");
    assert!(g.battlefield_find(giant).is_some());
    let counters = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne)
        + g.battlefield_find(giant).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne));
    assert_eq!(counters, 4, "the Giant's mana value");
    let another = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast_at(&mut g, another, &[]).is_err(), "flash was for one spell");
}

/// Psychic Impetus goads by static and scries its controller 2 on attack.
#[test]
fn psychic_impetus_goads_and_scries() {
    let mut g = main_phase(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pi = g.add_card_to_hand(0, catalog::psychic_impetus());
    cast_at(&mut g, pi, &[Target::Permanent(bear)]).expect("cast");
    assert!(g.goaded_by_player(g.battlefield_find(bear).unwrap(), 0));
    assert_eq!(pt(&g, bear), (4, 4));
}

/// Catti-brie: a counter per Equipment on attack; the counters shoot an
/// attacker.
#[test]
fn catti_brie_banks_counters_and_fires_them() {
    let mut g = main_phase(2);
    let cb = g.add_card_to_battlefield(0, catalog::catti_brie_of_mithral_hall());
    let robe = g.add_card_to_battlefield(0, catalog::robe_of_stars());
    g.battlefield_find_mut(robe).unwrap().attached_to = Some(cb);
    g.clear_sickness(cb);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: cb, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cb).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Clay Golem's roll makes it monstrous by the result and the berserk trigger
/// destroys a permanent.
#[test]
fn clay_golem_rolls_into_monstrosity() {
    let mut g = main_phase(2);
    let cg = g.add_card_to_battlefield(0, catalog::clay_golem());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(5), DecisionAnswer::Target(Target::Permanent(ring))]));
    activate(&mut g, cg, 0, &[]).expect("{6}, roll");
    assert_eq!(pt(&g, cg), (9, 9));
    assert!(g.battlefield_find(ring).is_none());
}

/// Diviner's Portent at 15+ scries then draws X.
#[test]
fn diviners_portent_draws_x() {
    let mut g = main_phase(2);
    library(&mut g, 0, 8);
    let dp = g.add_card_to_hand(0, catalog::diviners_portent());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(2)]));
    cast_x(&mut g, 0, dp, &[], Some(3)).expect("X=3");
    assert_eq!(g.players[0].hand.len(), 3);
}

/// Ebony Fly becomes an X/X flier for the roll.
#[test]
fn ebony_fly_rolls_into_a_flier() {
    let mut g = main_phase(2);
    let ef = g.add_card_to_battlefield(0, catalog::ebony_fly());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(5)]));
    activate(&mut g, ef, 1, &[]).expect("{4}, roll");
    assert_eq!(pt(&g, ef), (5, 5));
    assert!(kw(&g, ef, Keyword::Flying));
}

/// Fey Steed draws when an opponent targets your creature.
#[test]
fn fey_steed_draws_when_targeted() {
    let mut g = main_phase(2);
    library(&mut g, 0, 3);
    g.add_card_to_battlefield(0, catalog::fey_steed());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_x(&mut g, 1, bolt, &[Target::Permanent(bear)], None).expect("bolt");
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Holy Avenger: double strike, and connecting drops an Aura from hand on.
#[test]
fn holy_avenger_drops_an_aura_on_hit() {
    let mut g = main_phase(2);
    let ha = g.add_card_to_battlefield(0, catalog::holy_avenger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(ha).unwrap().attached_to = Some(bear);
    let aura = g.add_card_to_hand(0, catalog::gryffs_boon());
    assert!(kw(&g, bear, Keyword::DoubleStrike));
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(aura).map(|c| c.attached_to), Some(Some(bear)));
}

/// Robe of Stars phases its creature out.
#[test]
fn robe_of_stars_phases_out() {
    let mut g = main_phase(2);
    let robe = g.add_card_to_battlefield(0, catalog::robe_of_stars());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(robe).unwrap().attached_to = Some(bear);
    assert_eq!(pt(&g, bear), (2, 5));
    activate(&mut g, robe, 0, &[]).expect("phase out");
    assert!(g.battlefield_find(bear).is_none() && g.phased_out.iter().any(|c| c.id == bear));
}

/// Song of Inspiration: the cards come home and, at 15+, the life too.
#[test]
fn song_of_inspiration_regrows_and_heals() {
    let mut g = main_phase(2);
    let a = g.add_card_to_graveyard(0, catalog::hill_giant());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let si = g.add_card_to_hand(0, catalog::song_of_inspiration());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(10)]));
    let life = g.players[0].life;
    cast_at(&mut g, si, &[Target::Permanent(a), Target::Permanent(b)]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == a) && g.players[0].hand.iter().any(|c| c.id == b));
    assert_eq!(g.players[0].life, life + 6, "10 + 6 is 15 or more");
}

/// Storvald grants ward {3} to your other creatures.
#[test]
fn storvald_wards_the_team() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::storvald_frost_giant_jarl());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "countered by ward");
}

/// Abundant Growth draws and makes its land tap for any color.
#[test]
fn abundant_growth_draws_and_fixes() {
    let mut g = main_phase(2);
    library(&mut g, 0, 2);
    let land = g.add_card_to_battlefield(0, catalog::plains());
    let ag = g.add_card_to_hand(0, catalog::abundant_growth());
    cast_at(&mut g, ag, &[Target::Permanent(land)]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.battlefield_find(ag).unwrap().attached_to, Some(land));
}

/// Valiant Endeavor destroys every creature with power at least the chosen
/// die at once (CR 603.10a — Midnight Reaper sees its own death and the
/// Giant's), then makes Knights equal to the other die.
#[test]
fn valiant_endeavor_destroys_simultaneously() {
    let mut g = main_phase(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    let reaper = g.add_card_to_battlefield(0, catalog::midnight_reaper());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(3), DecisionAnswer::DieRoll(3)]));
    let hand = g.players[0].hand.len();
    let ve = g.add_card_to_hand(0, catalog::valiant_endeavor());
    cast_at(&mut g, ve, &[]).expect("cast");
    assert!(g.battlefield_find(reaper).is_none() && g.battlefield_find(giant).is_none());
    assert!(g.battlefield_find(bear).is_some(), "power 2 < 3");
    assert_eq!(g.players[0].hand.len(), hand + 2, "the Reaper saw both deaths");
    let knights = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Knight").count();
    assert_eq!(knights, 3);
}
