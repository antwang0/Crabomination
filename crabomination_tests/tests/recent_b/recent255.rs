//! Functionality tests for `catalog::sets::decks::recent255` (MKM Cases — 2nd batch).

use crabomination::card::{CardDefinition, CardType, CounterType, CreatureType, Subtypes};
use crabomination::catalog;
use crabomination::game::actions::cost_reduction_for_spell;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::{cost, w};

fn detective(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

fn solve_now(g: &mut crabomination::game::GameState) {
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    drain_stack(g);
}

fn is_solved(g: &crabomination::game::GameState, id: crabomination::card::CardId) -> bool {
    g.battlefield.iter().find(|c| c.id == id).map(|c| c.case_solved).unwrap_or(false)
}

/// Ransacked Lab discounts instants/sorceries and solves after four are cast.
#[test]
fn ransacked_lab_discounts_and_solves() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_ransacked_lab());
    let bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 1, "I/S cost {{1}} less");
    solve_now(&mut g);
    assert!(!is_solved(&g, case), "unsolved with 0 I/S cast");
    g.players[0].instants_or_sorceries_cast_this_turn = 4;
    solve_now(&mut g);
    assert!(is_solved(&g, case), "solved after casting 4 I/S");
}

/// Stashed Skeleton mints a suspected Skeleton on ETB; it stays unsolved while
/// that Skeleton lives and solves once none remain.
#[test]
fn stashed_skeleton_etb_and_solve() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_stashed_skeleton());
    g.fire_self_etb_triggers(case, 0);
    drain_stack(&mut g);
    let skele = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Skeleton" && c.controller == 0)
        .expect("Skeleton token minted");
    assert!(skele.suspected, "the Skeleton token is suspected");
    let skele_id = skele.id;
    solve_now(&mut g);
    assert!(!is_solved(&g, case), "unsolved while a suspected Skeleton lives");
    g.battlefield.retain(|c| c.id != skele_id);
    solve_now(&mut g);
    assert!(is_solved(&g, case), "solved once no suspected Skeletons remain");
}

/// Pilfered Proof counters entering Detectives and solves at three.
#[test]
fn pilfered_proof_counters_detectives_and_solves() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_pilfered_proof());
    let d1 = g.add_card_to_battlefield(0, detective("Sleuth A"));
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered {
        card_id: d1,
    }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(d1).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "entering Detective got a +1/+1 counter",
    );
    g.add_card_to_battlefield(0, detective("Sleuth B"));
    g.add_card_to_battlefield(0, detective("Sleuth C"));
    solve_now(&mut g);
    assert!(is_solved(&g, case), "solved with three Detectives");
}
