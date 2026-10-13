//! Commander: EDHREC ranks 1501-2000, three-card gaps (`decks::cmdr_rank2k_g`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

fn attack(g: &mut GameState, attackers: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Sublime Archangel gives every other creature exalted: a lone attacker
/// gets +1/+1 per exalted instance.
#[test]
fn sublime_archangel_stacks_exalted() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sublime_archangel());
    ready(&mut g, 0, catalog::grizzly_bears());
    let lone = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, &[lone], 1);
    assert_eq!(g.computed_permanent(lone).unwrap().power, 2 + 3, "the Archangel and both Bears");
}

/// Mask of Immolation arrives on its own Elemental, which can sacrifice
/// itself for a point of damage.
#[test]
fn mask_of_immolation_brings_a_body() {
    let mut g = pod(2);
    let mask = g.add_card_to_hand(0, catalog::mask_of_immolation());
    cast(&mut g, mask, None);
    let elemental = g.battlefield.iter().find(|c| c.definition.name == "Elemental").expect("token").id;
    assert_eq!(g.battlefield_find(mask).unwrap().attached_to, Some(elemental));
    let life = g.players[1].life;
    act(&mut g, GameAction::ActivateAbility {
        card_id: elemental,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert_eq!(g.players[1].life, life - 1);
    assert!(g.battlefield_find(elemental).is_none());
}

/// Sai of the Shinobi may jump onto a creature as it enters.
#[test]
fn sai_of_the_shinobi_follows_the_newcomer() {
    let mut g = pod(2);
    let sai = ready(&mut g, 0, catalog::sai_of_the_shinobi());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert_eq!(g.battlefield_find(sai).unwrap().attached_to, Some(bear));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// The Lonely Mountain's Dwarf costs {1} less per Equipment you control.
#[test]
fn the_lonely_mountain_discounts_per_equipment() {
    let mut g = pod(2);
    let mountain = ready(&mut g, 0, catalog::the_lonely_mountain());
    ready(&mut g, 0, catalog::sai_of_the_shinobi());
    ready(&mut g, 0, catalog::stormrider_rig());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mountain,
        // The {T}: {R} comes from the Mountain type, not a printed ability.
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{4}{R} less {2}");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Dwarf"), 1);
}

/// Aegis of the Legion's mentor also shields the mentored creature.
#[test]
fn aegis_of_the_legion_mentors_with_a_shield() {
    let mut g = pod(2);
    let big = ready(&mut g, 0, catalog::hill_giant());
    let small = ready(&mut g, 0, catalog::goblin_champion());
    let aegis = ready(&mut g, 0, catalog::aegis_of_the_legion());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: aegis, target: big }).expect("equip");
    drain_stack(&mut g);
    attack(&mut g, &[big, small], 1);
    let c = g.battlefield_find(small).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(c.counter_count(CounterType::Shield), 1);
}

/// Lyra makes an Angel at the end step only after three draws.
#[test]
fn lyra_counts_three_draws() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::lyra_tolarian_archangel());
    let thirst = g.add_card_to_hand(0, catalog::thirst_for_identity());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, thirst, None);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Angel"), 1);
}

/// Stronghold Overseer pumps shadow creatures and shrinks the rest.
#[test]
fn stronghold_overseer_splits_by_shadow() {
    let mut g = pod(2);
    let overseer = ready(&mut g, 0, catalog::stronghold_overseer());
    let augur = ready(&mut g, 0, catalog::augur_il_vec());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    act(&mut g, GameAction::ActivateAbility { card_id: overseer, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None });
    assert_eq!(g.computed_permanent(augur).unwrap().power, 2);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 1);
    assert_eq!(g.computed_permanent(overseer).unwrap().power, 6);
}

/// Lyna gives a creature a shadow counter.
#[test]
fn lyna_hands_out_shadow() {
    let mut g = pod(2);
    let lyna = ready(&mut g, 0, catalog::lyna_veil_of_vengeance());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    act(&mut g, GameAction::ActivateAbility { card_id: lyna, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None, mode: None });
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Shadow));
}

/// Urgoros draws when its victim has no hand to discard from.
#[test]
fn urgoros_draws_off_an_empty_hand() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let urgoros = ready(&mut g, 0, catalog::urgoros_the_empty_one());
    g.players[1].hand.clear();
    let hand = g.players[0].hand.len();
    attack(&mut g, &[urgoros], 1);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Flitterwing Nuisance's activation makes this turn's combat damage draw.
#[test]
fn flitterwing_nuisance_turns_on_combat_draws() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let nuisance = g.add_card_to_hand(0, catalog::flitterwing_nuisance());
    cast(&mut g, nuisance, None);
    g.clear_sickness(nuisance);
    act(&mut g, GameAction::ActivateAbility { card_id: nuisance, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None });
    assert_eq!(g.computed_permanent(nuisance).unwrap().power, 2, "the -1/-1 counter is gone");
    let hand = g.players[0].hand.len();
    attack(&mut g, &[nuisance], 1);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}
