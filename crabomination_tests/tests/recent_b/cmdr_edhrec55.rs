//! Commander: EDHREC ranks 2001-2500, one-card gaps, first batch
//! (`decks::cmdr_edhrec55`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn fill_pool(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    fill_pool(g);
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Rasputin: a dream counter per opponent (three in a four-seat pod), a Goblin
/// for each opponent; one counter makes a Knight, the rest become {C}.
#[test]
fn rasputin_dreams_per_opponent() {
    let mut g = pod(4);
    let ras = g.add_card_to_hand(0, catalog::rasputin_the_oneiromancer());
    cast_at(&mut g, ras, None).expect("cast");
    assert_eq!(g.battlefield_find(ras).unwrap().counter_count(CounterType::Dream), 3);
    for seat in 1..4 {
        assert!(g.battlefield.iter().any(|c| c.controller == seat && c.definition.name == "Goblin"));
    }
    g.clear_sickness(ras);
    activate(&mut g, ras, 1);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Knight"));
    assert_eq!(g.battlefield_find(ras).unwrap().counter_count(CounterType::Dream), 2);
    g.battlefield_find_mut(ras).unwrap().tapped = false;
    g.players[0].mana_pool = Default::default();
    activate(&mut g, ras, 0);
    assert_eq!(g.players[0].mana_pool.total(), 2, "two {{C}} from the last two counters");
}

/// Sparring Construct's death puts a counter on your creature; Skeletal
/// Grimace gives +1/+1 and a regeneration ability (CR 701.15).
#[test]
fn sparring_construct_and_skeletal_grimace() {
    let mut g = pod(2);
    let construct = ready(&mut g, 0, catalog::sparring_construct());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.destroy_permanent(construct, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);

    let grimace = g.add_card_to_hand(0, catalog::skeletal_grimace());
    cast_at(&mut g, grimace, Some(Target::Permanent(bears))).expect("aura");
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4, "2 + counter + aura");
    assert_eq!(g.granted_abilities_for(bears).len(), 1);
}

/// Deepchannel Mentor makes blue creatures you control unblockable.
#[test]
fn deepchannel_mentor_hides_blue_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::deepchannel_mentor());
    let blue = ready(&mut g, 0, catalog::hookblade_veteran());
    let green = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(g.computed_permanent(blue).unwrap().keywords().contains(&Keyword::Unblockable));
    assert!(!g.computed_permanent(green).unwrap().keywords().contains(&Keyword::Unblockable));
}

/// Nightmare's Thirst counts all life gained this turn, its own point
/// included.
#[test]
fn nightmares_thirst_scales_with_life_gained() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.players[0].life_gained_this_turn = 2;
    let spell = g.add_card_to_hand(0, catalog::nightmares_thirst());
    cast_at(&mut g, spell, Some(Target::Permanent(giant))).expect("cast");
    assert!(g.battlefield_find(giant).is_none(), "3/3 got -3/-3");
}

/// Visions of Phyrexia: a Powerstone at the end step unless a card was played
/// from exile this turn.
#[test]
fn visions_of_phyrexia_makes_powerstones_when_unused() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::visions_of_phyrexia());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let stones = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Powerstone").count();
    assert_eq!(stones(&g), 1);
    assert!(g.battlefield.iter().find(|c| c.definition.name == "Powerstone").unwrap().tapped);
    g.players[0].spells_cast_from_exile_this_turn = 1;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(stones(&g), 1, "played from exile: no Powerstone");
}

/// Kamahl's Will with a commander out does both modes: the lands animate,
/// then every creature you control hits the target.
#[test]
fn kamahls_will_does_both_with_a_commander() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    g.seat_commanders(0, vec![catalog::grizzly_bears()]);
    let cmdr = g.players[0].command.pop().expect("commander");
    let cmdr_id = cmdr.id;
    g.battlefield.push(cmdr);
    g.clear_sickness(cmdr_id);
    g.add_card_to_battlefield(0, catalog::forest());
    let wurm = ready(&mut g, 1, catalog::colossal_dreadmaw());
    let will = g.add_card_to_hand(0, catalog::kamahls_will());
    fill_pool(&mut g);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: will,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(wurm)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("both modes with a commander out");
    drain_stack(&mut g);
    // Two 2/2 Bears and a 1/1 Forest: five damage, not enough for a 6/6.
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 5);
}

/// Cosmic Cube casts a free spell off the top when you attack, capped by the
/// greatest attacking power.
#[test]
fn cosmic_cube_casts_off_the_top_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = pod(2);
    ready(&mut g, 0, catalog::cosmic_cube());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::sparring_construct());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::colossal_dreadmaw());
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sparring Construct"), "the {{1}} spell, not a Dreadmaw");
    assert_eq!(g.players[0].library.len(), 5);
}
