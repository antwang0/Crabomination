//! Functionality tests for `catalog::sets::decks::recent174`.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;

/// Boom Scholar's exhaust gives your team trample and grows itself by two.
#[test]
fn boom_scholar_exhaust_team_trample() {
    let mut g = two_player_game();
    let scholar = g.add_card_to_battlefield(0, catalog::boom_scholar());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(scholar);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: scholar, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("exhaust");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample),
        "team gained trample");
    assert_eq!(g.battlefield_find(scholar).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Spire Mechcycle's exhaust taps another Vehicle to animate and grows by the
/// number of other Mounts/Vehicles you control.
#[test]
fn spire_mechcycle_exhaust_scales_counters() {
    use crate::card::CardType;
    let mut g = two_player_game();
    let cycle = g.add_card_to_battlefield(0, catalog::spire_mechcycle());
    let helper = g.add_card_to_battlefield(0, catalog::skybox_ferry()); // another Vehicle
    let helper2 = g.add_card_to_battlefield(0, catalog::veloheart_bike()); // and another
    g.clear_sickness(cycle);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cycle, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("exhaust taps another Vehicle");
    drain_stack(&mut g);
    assert!(g.computed_permanent(cycle).unwrap().card_types.contains(&CardType::Creature), "animated");
    // Two other Mounts/Vehicles → two counters.
    assert_eq!(g.battlefield_find(cycle).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    // One of the helpers was tapped to pay the cost.
    assert!(g.battlefield_find(helper).unwrap().tapped || g.battlefield_find(helper2).unwrap().tapped,
        "a helper Vehicle was tapped");
}

/// Slick Imitator copies your spell at max speed (opponent eats two Bolts).
#[test]
fn slick_imitator_copies_spell_at_max_speed() {
    let mut g = two_player_game();
    let imitator = g.add_card_to_battlefield(0, catalog::slick_imitator());
    g.clear_sickness(imitator);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].speed = 4;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let opp_life = g.players[1].life;
    // Cast the Bolt at the opponent; it sits on the stack.
    g.cast_spell(bolt, Some(Target::Player(1)), vec![], None, None).expect("cast Bolt");
    // Copy it with Slick Imitator's max-speed sacrifice ability.
    g.perform_action(GameAction::ActivateAbility {
        card_id: imitator, ability_index: 0, target: Some(Target::Permanent(bolt)),
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("copy the Bolt at max speed");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 6, "two Bolts resolved (original + copy)");
    assert!(g.battlefield_find(imitator).is_none(), "sacrificed to copy");
}
