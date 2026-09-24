//! Commander: the Nature of the Beast precon (C13, Marath,
//! `decks::cmdr_marath`).

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

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target], x: Option<u32>, mode: Option<usize>) -> Result<(), String> {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: x,
        mode,
    })
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0)
}

fn attack_with(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for a in attackers {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// Marath enters with a counter per mana spent and spends X of them on its
/// modes.
#[test]
fn marath_spends_its_counters() {
    let mut g = main_phase(2);
    let m = g.add_card_to_hand(0, catalog::marath_will_of_the_wild());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(plus(&g, m), 3);
    let life = g.players[1].life;
    activate(&mut g, m, 0, &[Target::Player(1)], Some(2), Some(1)).expect("X = 2 damage");
    assert_eq!(g.players[1].life, life - 2);
    assert_eq!(plus(&g, m), 1);
}

/// Curse of Chaos lets a player attacking the cursed one loot.
#[test]
fn curse_of_chaos_loots_the_attacker() {
    let mut g = main_phase(3);
    g.add_card_to_library(0, catalog::plains());
    let c = g.add_card_to_hand(0, catalog::curse_of_chaos());
    cast_at(&mut g, c, &[Target::Player(1)]).expect("cast");
    let pitched = g.add_card_to_hand(0, catalog::mountain());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_with(&mut g, &[bear], 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitched));
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Fiery Justice splits 5 damage, and an opponent gains 5.
#[test]
fn fiery_justice_divides_five() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(1, catalog::serra_angel());
    let life = g.players[1].life;
    let fj = g.add_card_to_hand(0, catalog::fiery_justice());
    cast_at(&mut g, fj, &[Target::Permanent(a)]).expect("cast");
    assert!(g.battlefield_find(a).is_none(), "all five on the Angel");
    assert_eq!(g.players[1].life, life + 5);
}

/// From the Ashes: nonbasic lands die; each player fetches a basic per land.
#[test]
fn from_the_ashes_trades_nonbasics_for_basics() {
    let mut g = main_phase(2);
    let nb = g.add_card_to_battlefield(1, catalog::festering_thicket());
    let basic = g.add_card_to_battlefield(1, catalog::forest());
    let lib = g.add_card_to_library(1, catalog::swamp());
    let fa = g.add_card_to_hand(0, catalog::from_the_ashes());
    cast_at(&mut g, fa, &[]).expect("cast");
    assert!(g.battlefield_find(nb).is_none() && g.battlefield_find(basic).is_some());
    assert!(g.battlefield_find(lib).is_some(), "the opponent fetched a basic");
}

/// Gahiji pumps any creature attacking one of your opponents.
#[test]
fn gahiji_pumps_attacks_on_your_opponents() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::gahiji_honored_one());
    g.active_player_idx = 1;
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack_with(&mut g, &[theirs], 2);
    assert_eq!(pt(&g, theirs), (4, 2), "attacking seat 2, an opponent of Gahiji's");
}

/// Magus of the Arena taps both and makes them fight.
#[test]
fn magus_of_the_arena_forces_a_fight() {
    let mut g = main_phase(2);
    let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_arena());
    g.clear_sickness(magus);
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, magus, 0, &[Target::Permanent(angel), Target::Permanent(bear)], None, None).expect("activate");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(angel).unwrap().tapped);
}

/// Mayael puts a power-5 creature from the top five onto the battlefield.
#[test]
fn mayael_finds_a_fatty() {
    let mut g = main_phase(2);
    let big = g.add_card_to_library(0, catalog::rakeclaw_gargantuan());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    let m = g.add_card_to_battlefield(0, catalog::mayael_the_anima());
    g.clear_sickness(m);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![big])]));
    activate(&mut g, m, 0, &[], None, None).expect("activate");
    assert!(g.battlefield_find(big).is_some());
}

/// Mystic Barrier points every attack at the nearest opponent one way.
#[test]
fn mystic_barrier_restricts_attacks() {
    let mut g = main_phase(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let mb = g.add_card_to_hand(0, catalog::mystic_barrier());
    cast_at(&mut g, mb, &[]).expect("cast");
    assert_eq!(g.attackable_players_for(0), vec![1], "left is the next seat");
}

/// Naya Soulbeast grows by the top cards' mana values.
#[test]
fn naya_soulbeast_counts_the_top_cards() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::serra_angel());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let nb = g.add_card_to_hand(0, catalog::naya_soulbeast());
    cast_at(&mut g, nb, &[]).expect("cast");
    assert_eq!(plus(&g, nb), 7, "5 + 2");
}

/// Rain of Thorns picks any of its three modes.
#[test]
fn rain_of_thorns_hits_three_kinds() {
    let mut g = main_phase(2);
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let ench = g.add_card_to_battlefield(1, catalog::witch_hunt());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let rt = g.add_card_to_hand(0, catalog::rain_of_thorns());
    act(&mut g, GameAction::CastSpellSpree {
        card_id: rt,
        spree_modes: vec![0, 1, 2],
        target: Some(Target::Permanent(art)),
        additional_targets: vec![Target::Permanent(ench), Target::Permanent(land)],
        x_value: None,
    })
    .expect("cast");
    assert!(g.battlefield_find(art).is_none() && g.battlefield_find(ench).is_none() && g.battlefield_find(land).is_none());
}

/// Rakeclaw gives a big creature first strike; not a small one.
#[test]
fn rakeclaw_grants_first_strike_to_big_creatures() {
    let mut g = main_phase(2);
    let rk = g.add_card_to_battlefield(0, catalog::rakeclaw_gargantuan());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(activate(&mut g, rk, 0, &[Target::Permanent(bear)], None, None).is_err());
    activate(&mut g, rk, 0, &[Target::Permanent(rk)], None, None).expect("itself");
    assert!(g.computed_permanent(rk).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// Spawning Grounds turns a land into a Beast factory.
#[test]
fn spawning_grounds_makes_beasts() {
    let mut g = main_phase(2);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let sg = g.add_card_to_hand(0, catalog::spawning_grounds());
    cast_at(&mut g, sg, &[Target::Permanent(land)]).expect("cast");
    // The granted ability follows the Forest's own mana ability.
    activate(&mut g, land, 1, &[], None, None).expect("the granted ability");
    let beasts = named(&g, 0, "Beast");
    assert_eq!(pt(&g, beasts[0]), (5, 5));
}

/// Terra Ravager counts the defending player's lands.
#[test]
fn terra_ravager_counts_defending_lands() {
    let mut g = main_phase(2);
    for _ in 0..4 {
        g.add_card_to_battlefield(1, catalog::forest());
    }
    let tr = g.add_card_to_battlefield(0, catalog::terra_ravager());
    attack_with(&mut g, &[tr], 1);
    assert_eq!(pt(&g, tr), (4, 4));
}

/// Where Ancients Tread fires for a power-5 creature entering.
#[test]
fn where_ancients_tread_burns_on_a_big_arrival() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::where_ancients_tread());
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Target(Target::Player(1))]));
    let rk = g.add_card_to_hand(0, catalog::rakeclaw_gargantuan());
    cast_at(&mut g, rk, &[]).expect("cast");
    assert_eq!(g.players[1].life, life - 5);
}

/// CR 119.7 — Witch Hunt stops life gain and burns its controller.
#[test]
fn cr_119_7_witch_hunt_stops_life_gain() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::witch_hunt());
    let life = g.players[1].life;
    let fj = g.add_card_to_hand(0, catalog::fiery_justice());
    let a = g.add_card_to_battlefield(1, catalog::serra_angel());
    cast_at(&mut g, fj, &[Target::Permanent(a)]).expect("cast");
    assert_eq!(g.players[1].life, life, "no gain under Witch Hunt");
}

/// The bot spends Marath's counters: with mana up and a target around, it
/// activates the {X} ability.
#[test]
fn bot_spends_maraths_counters() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = main_phase(2);
    let m = g.add_card_to_battlefield(0, catalog::marath_will_of_the_wild());
    g.battlefield_find_mut(m).unwrap().add_counters(CounterType::PlusOnePlusOne, 5);
    g.clear_sickness(m);
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let mut bot = HeuristicBot::new();
    let mut used = false;
    for _ in 0..20 {
        g.priority.player_with_priority = 0;
        let Some(action) = bot.next_action(&g, 0) else { break };
        if matches!(action, GameAction::ActivateAbility { card_id, .. } if card_id == m) {
            used = true;
            break;
        }
        let pass = matches!(action, GameAction::PassPriority);
        let _ = g.perform_action(action);
        drain_stack(&mut g);
        if pass {
            break;
        }
    }
    assert!(used, "the bot never spent Marath's counters");
}
