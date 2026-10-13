//! Commander: Braids', Taigam's, Carth's, Gogo's, Edea's, Uugguu's and Loot's
//! last gaps (`decks::cmdr_edhrec40`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 500.8 — Shadow of the Second Sun on an opponent: their postcombat main
/// banks an extra beginning phase; its caster's own doesn't.
#[test]
fn cr_500_8_shadow_of_the_second_sun_follows_the_enchanted_player() {
    let mut g = pod(3);
    let shadow = g.add_card_to_hand(0, catalog::shadow_of_the_second_sun());
    cast(&mut g, shadow, Some(Target::Player(2))).expect("shadow");
    g.step = TurnStep::PostCombatMain;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.additional_beginning_phases, 0, "not the enchanted player's turn");
    g.active_player_idx = 2;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.additional_beginning_phases, 1);
}

/// Jhoira's Timebug: its controller picks — a vanishing permanent of theirs
/// gains a time counter or loses one.
#[test]
fn jhoiras_timebug_moves_a_time_counter_either_way() {
    let mut g = pod(2);
    let bug = g.add_card_to_battlefield(0, catalog::jhoiras_timebug());
    let clock = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(clock).unwrap().counters.insert(CounterType::Time, 2);
    for (answer, want) in [(1, 3), (0, 2)] {
        g.battlefield_find_mut(bug).unwrap().tapped = false;
        g.battlefield_find_mut(bug).unwrap().summoning_sick = false;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(answer)]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: bug,
            ability_index: 0,
            target: Some(Target::Permanent(clock)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("timebug");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(clock).unwrap().counter_count(CounterType::Time), want);
    }
}

/// CR 606.1 — a loyalty ability is an activated ability: Way of the Paradox
/// sees Vraska's +2 (a life and a land play); Way of the Necromancer's Jace
/// and Vraska each gain loyalty when a creature of yours dies.
#[test]
fn cr_606_1_the_ways_see_loyalty_activations_and_deaths() {
    let mut g = pod(2);
    let necro = g.add_card_to_hand(0, catalog::way_of_the_necromancer());
    cast(&mut g, necro, None).expect("necromancer");
    let jace = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Jace").map(|c| c.id).expect("Jace");
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 2);
    let paradox = g.add_card_to_hand(0, catalog::way_of_the_paradox());
    cast(&mut g, paradox, None).expect("paradox");
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 7, "empower 5 on the same Jace");
    let vraska = g.add_card_to_battlefield(0, catalog::vraska_relic_seeker());
    let life = g.players[0].life;
    loyalty(&mut g, vraska, 0, None).expect("+2");
    assert_eq!(g.players[0].life, life + 1, "Paradox: 1 life");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Pirate"));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(bear))).expect("shock");
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 8);
    assert_eq!(g.battlefield_find(vraska).unwrap().counter_count(CounterType::Loyalty), 6 + 2 + 1);
}

/// Vraska, Relic Seeker's −10 sets a player's life total to 1 (CR 119.5).
#[test]
fn cr_119_5_vraska_ultimate_sets_life_to_one() {
    let mut g = pod(3);
    let vraska = g.add_card_to_battlefield(0, catalog::vraska_relic_seeker());
    g.battlefield_find_mut(vraska).unwrap().counters.insert(CounterType::Loyalty, 10);
    loyalty(&mut g, vraska, 2, Some(Target::Player(2))).expect("-10");
    assert_eq!(g.players[2].life, 1);
    assert!(g.battlefield_find(vraska).is_none(), "zero loyalty: to the graveyard");
}
