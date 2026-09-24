//! Commander: the Token Triumph starter deck (SCD, Emmara, `decks::cmdr_emmara`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn counters(g: &GameState, id: CardId) -> u32 {
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

#[test]
fn camaraderie_scales_with_your_creatures() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::plains());
    }
    let life = g.players[0].life;
    let c = g.add_card_to_hand(0, catalog::camaraderie());
    cast(&mut g, c, &[]);
    assert_eq!(g.players[0].life, life + 3);
    assert_eq!(g.players[0].hand.len(), 3);
}

/// CR 509.1b — a blocker with less power than the Champion can't block.
#[test]
fn champion_of_lambholt_grows_and_bars_small_blockers() {
    let mut g = main_phase(2);
    let champ = g.add_card_to_battlefield(0, catalog::champion_of_lambholt());
    for _ in 0..2 {
        let b = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast(&mut g, b, &[]);
    }
    assert_eq!(pt(&g, champ), (3, 3));
    let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears" && c.controller == 0).unwrap().id;
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::celestial_force());
    attack_with(&mut g, &[bear], 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(small, bear)])).is_err(), "2 < 3");
    g.perform_action(GameAction::DeclareBlockers(vec![(big, bear)])).expect("7 >= 3");
}

#[test]
fn citywide_bust_kills_only_the_big() {
    let mut g = main_phase(2);
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::celestial_force());
    let c = g.add_card_to_hand(0, catalog::citywide_bust());
    cast(&mut g, c, &[]);
    assert!(g.battlefield_find(small).is_some());
    assert!(g.battlefield_find(big).is_none());
}

#[test]
fn dauntless_escort_makes_the_team_indestructible() {
    let mut g = main_phase(2);
    let e = g.add_card_to_battlefield(0, catalog::dauntless_escort());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, e, 0, None).expect("sacrifice");
    assert!(g.battlefield_find(e).is_none());
    assert!(g.permanent_has_keyword(bear, &Keyword::Indestructible));
}

#[test]
fn emmara_makes_a_soldier_when_tapped() {
    let mut g = main_phase(2);
    let e = g.add_card_to_battlefield(0, catalog::emmara_soul_of_the_accord());
    attack_with(&mut g, &[e], 1);
    assert_eq!(count_named(&g, 0, "Soldier"), 1);
    let s = g.battlefield.iter().find(|c| c.definition.name == "Soldier").unwrap().id;
    assert!(g.permanent_has_keyword(s, &Keyword::Lifelink));
}

#[test]
fn harvest_season_fetches_per_tapped_creature() {
    let mut g = main_phase(2);
    for _ in 0..2 {
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(b).unwrap().tapped = true;
    }
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let h = g.add_card_to_hand(0, catalog::harvest_season());
    cast(&mut g, h, &[]);
    assert_eq!(count_named(&g, 0, "Forest"), 2);
}

#[test]
fn jade_mage_makes_saprolings() {
    let mut g = main_phase(2);
    let j = g.add_card_to_battlefield(0, catalog::jade_mage());
    activate(&mut g, j, 0, None).expect("activate");
    assert_eq!(count_named(&g, 0, "Saproling"), 1);
}

/// It taps itself and another untapped creature for one mana.
#[test]
fn jaspera_sentinel_taps_a_friend_for_mana() {
    let mut g = main_phase(2);
    let j = g.add_card_to_battlefield(0, catalog::jaspera_sentinel());
    g.clear_sickness(j);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: j, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

#[test]
fn leafkin_druid_doubles_with_four_creatures() {
    let mut g = main_phase(2);
    let d = g.add_card_to_battlefield(0, catalog::leafkin_druid());
    g.clear_sickness(d);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: d, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// Lieutenant: with your commander out, a counter on each creature at combat.
#[test]
fn loyal_guardian_rewards_a_fielded_commander() {
    let mut g = main_phase(2);
    let lg = g.add_card_to_battlefield(0, catalog::loyal_guardian());
    let cmd = g.add_card_to_battlefield(0, catalog::emmara_soul_of_the_accord());
    g.players[0].commanders.push(cmd);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    assert_eq!(counters(&g, lg), 1);
    assert_eq!(counters(&g, cmd), 1);
}

#[test]
fn maja_pumps_the_team_and_makes_warriors_on_landfall() {
    let mut g = main_phase(2);
    let m = g.add_card_to_battlefield(0, catalog::maja_bretagard_protector());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, bear), (3, 3));
    assert_eq!(pt(&g, m), (2, 3), "other creatures only");
    let land = g.add_card_to_hand(0, catalog::plains());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Human Warrior"), 1);
}

#[test]
fn march_of_the_multitudes_makes_x_soldiers() {
    let mut g = main_phase(2);
    let m = g.add_card_to_hand(0, catalog::march_of_the_multitudes());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: m, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Soldier"), 3);
}

#[test]
fn presence_of_gond_grants_an_elf_maker() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let p = g.add_card_to_hand(0, catalog::presence_of_gond());
    cast(&mut g, p, &[Target::Permanent(bear)]);
    activate(&mut g, bear, 0, None).expect("granted ability");
    assert_eq!(count_named(&g, 0, "Elf Warrior"), 1);
}

/// Other creatures +1/+1, two lifelink Soldiers, and stolen creatures go home
/// at your end step.
#[test]
fn trostani_discordant_pumps_and_repatriates() {
    let mut g = main_phase(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().controller = 1;
    let t = g.add_card_to_hand(0, catalog::trostani_discordant());
    cast(&mut g, t, &[]);
    assert_eq!(count_named(&g, 0, "Soldier"), 2);
    let s = g.battlefield.iter().find(|c| c.definition.name == "Soldier").unwrap().id;
    assert_eq!(pt(&g, s), (2, 2));
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().controller, 0);
}

#[test]
fn valor_in_akros_pumps_on_each_arrival() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::valor_in_akros());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, b2, &[]);
    assert_eq!(pt(&g, bear), (3, 3));
}
