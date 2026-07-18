//! Comprehensive-Rules conformance for MKM's Solve keyword action (the Case
//! mechanic), multi-source deathtouch (CR 702.2c) through
//! `EachControlledCreatureDealsDamage`, and layer-7 P/T stacking (CR 613.7c/d).

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

fn is_solved(g: &crabomination::game::GameState, id: crabomination::card::CardId) -> bool {
    g.battlefield.iter().find(|c| c.id == id).map(|c| c.case_solved).unwrap_or(false)
}

/// A Case is solved only at *its controller's* end step, and once solved it
/// stays solved even if the condition later stops holding.
#[test]
fn cr_715_case_solves_at_controllers_end_step_and_persists() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    assert!(g.players[0].hand.is_empty(), "solve condition (empty hand) holds");

    // The opponent's end step must not solve the controller's Case.
    g.active_player_idx = 1;
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    assert!(!is_solved(&g, case), "not solved on the opponent's end step");

    // The controller's end step solves it.
    g.active_player_idx = 0;
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    drain_stack(&mut g);
    assert!(is_solved(&g, case), "solved at the controller's end step");

    // Once solved, a later end step where the condition fails leaves it solved.
    g.add_card_to_hand(0, catalog::forest());
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    assert!(is_solved(&g, case), "solved state persists");
}

/// CR 702.2c — a deathtouch source dealing any nonzero damage is lethal. When
/// several of your creatures each ping a target, the deathtouch pinger's damage
/// is enough to destroy it even though its share is only 1.
#[test]
fn cr_702_2c_multi_source_ping_deathtouch_is_lethal() {
    let mut g = two_player_game();
    // A 0/5 wall survives 2 plain damage, but a deathtouch ping kills it.
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens()); // 0/4
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, plain
    let rats = g.add_card_to_battlefield(0, catalog::typhoid_rats()); // 1/1 deathtouch
    let effect = crabomination::effect::Effect::EachControlledCreatureDealsDamage {
        to: crabomination::effect::shortcut::target_filtered(
            crabomination::card::SelectionRequirement::Creature,
        ),
        amount: crabomination::effect::Value::ONE,
    };
    let ctx = EffectContext::for_ability(rats, 0, Some(Target::Permanent(wall)));
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none(), "deathtouch ping among the group is lethal");
}

/// CR 613.7c/d — a +1/+1 anthem (layer 7c) and a +1/+1 counter (layer 7d) both
/// apply to a creature's power and toughness.
#[test]
fn cr_613_7_anthem_and_counter_both_apply() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_battlefield(0, catalog::gaeas_anthem()); // +1/+1 to your creatures
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "2/2 + anthem + counter = 4/4");
}
